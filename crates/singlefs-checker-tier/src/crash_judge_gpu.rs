//! GPU 判器（用户 2026-09-28 定「GPU 判器现在实现、整套做完」）：把 `crate::crash_judge_tables` 摊出来的表整份上卡，
//! 每个调用判一个崩溃状态——恢复（看 journal 与不看 journal 各一遍：择系统配置、择根、扫环重放、走到第一个文件）、
//! 七类 oracle、记录核对器两条判据，判法与 CPU 判器（`crate::crash::judge_crash_image`）逐条相同；GPU 与 CPU 逐状态比是它的判别力自证。
//! 内核分五份源码：`crash_judge_common.wgsl`（各内核共用的解析与查表）、`crash_judge_recovery.wgsl`（恢复、oracle、记录核对器）、
//! `crash_judge_checker_common.wgsl`（池级 checker 两个内核共用的读法与交接段）、`crash_judge_checker.wgsl`（走读内核：第一、二批不变量，
//! 连同里程碑三二·1 那一批里的十五条、I-9.27 与 I-3.5，与里程碑三·3 墓碑树、谱系树条目身份那两条 I-9.17、I-9.23）、`crash_judge_checker_scan.wgsl`
//! （扫描内核：第三批、I-3.12、I-5.5、I-9.28，与里程碑三·3 删除线的删除态扫描、I-9.6 墓碑那一半、墓碑与谱系那一遍）；判不了的状态（表越过内核容量）置最高位，
//! 放量路径（`crate::crash_amplification` 收卡上结论那一段）遇到它就让整趟停下、报哪张表满了，配置开了 GPU 不退回 CPU 判器。
//!
//! 一个状态的结论 [`VERDICT_WORDS`] 个 u32：第 0 字是红位（位 0..6 看 journal 那一遍 oracle 的七类、位 8..14 不看那一遍、
//! 位 16 根在而记录都不在、位 17 恢复自称的状态少单元、位 31 判不了）；第 1 字判不了的原因（判得了时低 16 位是看 journal 那一遍
//! 重放停在哪一道、高 16 位是记录数，诊断用）；第 2 / 3 字看 journal 那一遍落到的
//! 实例代号与 checkpoint_txg 低 32 位；第 4 / 5 字它的走读结局与失败点（失败点码是 [`gpu_failure_point_code`] 那一张表）；第 6 / 7 字不看那一遍的结局与失败点；
//! 第 8..12 字池级 checker 的违例位（[`POOL_CHECKER_VIOLATED_WORDS`] 个字，128 位）；第 12 字池级 checker 走到第几步；第 13 字它判不了的原因；
//! 第 14 / 15 字不看 journal 那一遍落到的实例代号与 checkpoint_txg 低 32 位（判决 `research/prompts/m3-crash-enumeration-merge-r2-main-verification.md`
//! 第二节 L1-6）；第 16..20 字池级 checker 逐条不变量的「判到」位（成立或违例，与 CPU 计数里「评估过」同一口径；同一判决 L1-4）。
//! 这几个字号、打包表的表号与段表宽、段的展开方式码、失败点码在内核里的常量由 [`kernel_prelude_wgsl_constants`] 从 Rust 生成、
//! 接在每个内核源码前面，不在 WGSL 里另写一份。对拍比的是 [`CrossCheckWords`]：GPU 与 CPU 两侧各由一个构造函数换出来。
//! 判法代码的摘要 [`gpu_judge_digest`] 进判定表的键：内核改一个字，旧行留着当历史，新行另起。
//!
//! 这一份只放判法：内核原文、表怎么打包、派活参数与结论各是哪几个字、草稿区每个状态多大。怎么上卡、一次派活带几个状态、
//! 几张卡怎么用，在 `crate::crash_judge_dispatch`——那一份不在判法闭包里（用户 2026-09-29 定「派活和判法拆出来最好」），
//! 改它不换判法版本；这一份不许引它。
use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
use singlefs_checker::walk::ContainerCapacityKnobs;
use singlefs_core::recovery::{RecoveryFailure, RecoveryOutcome};
use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};

use crate::crash::{Layer0OracleViolationKind, Layer0StateJudgement};
use crate::crash_facts::segment_expansion_code_wgsl_constants;
use crate::crash_judge_tables::{JudgeTablesWords, JUDGE_TABLE_COUNT, SEGMENT_WORDS};

/// 两个内核共用的那一半源码：表的读法、u64、落点与版本、指针与单元的解析、系统配置与根与 journal 的解析、掩码。
pub const GPU_JUDGE_COMMON_SOURCE: &str = include_str!("crash_judge_common.wgsl");
/// 恢复内核：两遍恢复、oracle、记录核对器（写每个状态结论的第 0..8 字）。
pub const GPU_JUDGE_RECOVERY_SOURCE: &str = include_str!("crash_judge_recovery.wgsl");
/// 池级 checker 两个内核共用的那一半：不变量下标、根与记录的读法、草稿区各集合的位置、交接段。
pub const GPU_JUDGE_CHECKER_COMMON_SOURCE: &str = include_str!("crash_judge_checker_common.wgsl");
/// 池级 checker 走读内核：第一、二批不变量的违例位（写违例位那几个字、走到第几步与判不了的原因），走完把计数与几何交进草稿区的交接段。
pub const GPU_JUDGE_CHECKER_SOURCE: &str = include_str!("crash_judge_checker.wgsl");
/// 池级 checker 扫描内核：第三批（扫描方向与记账那九条），违例位并进违例位那几个字。
pub const GPU_JUDGE_CHECKER_SCAN_SOURCE: &str = include_str!("crash_judge_checker_scan.wgsl");

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
/// 每个状态的结论几个 u32：恢复内核 8 个，池级 checker 内核违例位、走到第几步、判不了的原因，不看 journal 那一遍落到的根，判到位。
pub const VERDICT_WORDS: usize = POOL_CHECKER_JUDGED_FIRST_WORD + POOL_CHECKER_JUDGED_WORDS;
/// 走读结局的码（第 4、6 字；内核里的 `OUTCOME_*`）。
pub const RECOVERY_OUTCOME_CODE_NO_FILE: u32 = 0;
pub const RECOVERY_OUTCOME_CODE_FILE_READ: u32 = 1;
pub const RECOVERY_OUTCOME_CODE_FAILED: u32 = 2;

// 违例位装得下池级 checker 的每一条不变量；读出来的是一个 u128，字数再多就装不进。
const _: () = assert!(
    singlefs_checker::image::IMPLEMENTED_INVARIANTS.len() <= POOL_CHECKER_VIOLATED_BIT_CAPACITY,
    "池级 checker 的不变量多于违例位的容量：先加宽 POOL_CHECKER_VIOLATED_WORDS"
);
const _: () = assert!(
    POOL_CHECKER_VIOLATED_BIT_CAPACITY <= 128,
    "违例位读成一个 u128"
);

/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：全部 92 条——三批 49 条，加里程碑三二·1 那一批 16 条
/// （判决 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行），下标 65 的 I-3.5（引用区间的精确性，
/// 走读内核在回退候选集的版本都走完之后逐对判），再加 C673（三 oracle 合谋测试的补 oracle 改法没做）⑤ ⑥ 那两条
/// （下标 66 是 I-9.27，走读内核挨着 I-9.16 按表判；下标 67 是 I-5.5，扫描内核每一版重读树表与 extent 树判），
/// 再加下标 68 的 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同，扫描内核挨着 I-9.14 读根表里每一条根的树表判；
/// 部分没判与不适用一样不置违例位，结论里只有违例位），再加里程碑三·3 删除线那五条：下标 69 的 I-9.5、70 的 I-3.4、71 的 I-4.7、
/// 72 的 I-4.6 由扫描内核的删除态扫描在回退候选集的每一版上判（判哪几版、经头 ID 配对、读不全不置位，照 `singlefs_checker::walk` 的
/// `judge_deletion_state_of_the_versions`），73 的 I-4.4 第一版池级 checker 恒报不适用、内核不置位；I-9.6 墓碑那一半（水位 > 所选根墓碑树里
/// 已发布墓碑的最大对象 ID）也在扫描内核里判、并进下标 41 那一位。对拍逐位比、不屏蔽。
/// 下标 74–82 是里程碑三·3 墓碑与谱系那九条（I-9.17、I-9.18、I-9.19、I-9.20、I-9.21、I-9.22、I-9.23、I-9.24、I-9.29，CPU 第 22 批、GPU 第 23 批）：
/// 走读内核在墓碑树、谱系树的每条内部条目上判 I-9.17、I-9.23（身份不符置位、不记走读失败；孩子是哪一类按它头里的类标签认、不按类型段定读多长，
/// 头里是另一类、按那一类的长度自证得过的按它自己那一类登记引用、不往下走，第 24 批跟 CPU 同改，inode 树内部条目的 I-9.2 同改）；
/// 扫描内核的删除态扫描读队列时判 I-9.24，
/// 墓碑与谱系那一遍在同一份版本清单上判 I-9.18、I-9.19、I-9.20、I-9.21（没装旋钮那一支）与 I-9.10 墓碑那一个等号（并进下标 43），
/// I-9.29 收各版的号 → 对象出生代进一张跨版本的表判，最新根的实例表沿链读不出时那张表不比、不置位（CPU 报部分没判，第 24 批）；
/// 一棵树的容器、一版的谱系记录、跨版本那张表、两两比不重号的墓碑总数越过上限的判不了。
/// 下标 79 的 I-9.22 只在池级 checker 带旋钮值的入口上判，层 0 与放量走旧入口（`check_pool_image`，恒报不适用）、编的是三棵容器树都没装旋钮的
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
/// 用户 2026-10-04 定「改回要求全覆盖」：没有「内核还没判」那一张表。
pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
    74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91,
];

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
         const OUTCOME_NO_FILE: u32 = {RECOVERY_OUTCOME_CODE_NO_FILE}u;\n\
         const OUTCOME_FILE_READ: u32 = {RECOVERY_OUTCOME_CODE_FILE_READ}u;\n\
         const OUTCOME_FAILED: u32 = {RECOVERY_OUTCOME_CODE_FAILED}u;\n"
    )
}

/// 打包表的表数、表号与段表宽在内核里的常量（WGSL 源码）：表号就是 [`JudgeTablesWords::every_table_name`] 里的次序，名字大写、前面加 `T_`。
#[must_use]
pub fn judge_table_layout_wgsl_constants() -> String {
    let mut constants = format!(
        "const TABLE_COUNT: u32 = {JUDGE_TABLE_COUNT}u;\nconst SEGMENT_WORDS: u32 = {SEGMENT_WORDS}u;\n"
    );
    for (index, name) in JudgeTablesWords::every_table_name().iter().enumerate() {
        constants.push_str(&format!(
            "const T_{}: u32 = {index}u;\n",
            name.to_ascii_uppercase()
        ));
    }
    constants
}

/// 接在每个判器内核源码最前面的全部常量：结论字号、打包表的表号与段表宽、段的展开方式码、失败点码。
#[must_use]
pub fn kernel_prelude_wgsl_constants() -> String {
    format!(
        "{}{}{}{}",
        verdict_layout_wgsl_constants(),
        judge_table_layout_wgsl_constants(),
        segment_expansion_code_wgsl_constants(),
        failure_point_wgsl_constants()
    )
}

/// 失败点码一张表的单一来源：每一行一个常量名与它的码，生成 Rust 常量、[`GPU_FAILURE_POINTS`] 与 WGSL 常量（名字去掉 `GPU_` 前缀）。
macro_rules! gpu_failure_points {
    ($($constant:ident = $code:literal,)*) => {
        $(pub const $constant: u32 = $code;)*
        /// 全部失败点码（常量名、码），按码升序：[`failure_point_wgsl_constants`] 照它生成内核里的常量。
        pub const GPU_FAILURE_POINTS: &[(&str, u32)] = &[$((stringify!($constant), $constant),)*];
    };
}

gpu_failure_points! {
    GPU_FAILURE_POINT_NONE = 0,
    GPU_FAILURE_POINT_NO_VALID_SYSTEM_CONFIGURATION = 1,
    GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_INCOMPAT_BITS_NOT_RECOGNIZED = 2,
    GPU_FAILURE_POINT_SYSTEM_CONFIGURATIONS_DISAGREE = 3,
    GPU_FAILURE_POINT_NO_DEVICE_IN_THE_POOL = 4,
    GPU_FAILURE_POINT_ROOT_RING_SLOTS_PER_REGION_OUT_OF_RANGE = 5,
    GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_VALUE_REFUSED = 6,
    GPU_FAILURE_POINT_DEVICE_ENDS_BEFORE_THE_UNIT_AREA_START = 7,
    GPU_FAILURE_POINT_NO_VALID_ROOT = 8,
    GPU_FAILURE_POINT_UNIT_UNREADABLE = 9,
    GPU_FAILURE_POINT_UNIT_MALFORMED = 10,
    GPU_FAILURE_POINT_INVARIANT_VIOLATED = 11,
    GPU_FAILURE_POINT_ENTRY_NARROWER_THAN_ITS_FIELD_TABLE = 12,
    GPU_FAILURE_POINT_INODE_NUMBER_WATERMARK_ROW_MISSING_FROM_THE_ACCOUNTING_TREE = 13,
    GPU_FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY = 14,
    GPU_FAILURE_POINT_MAPPING_MISS = 15,
    GPU_FAILURE_POINT_MAPPING_STILL_UNREADABLE = 16,
    GPU_FAILURE_POINT_ROOT_PUBLISH_CARRIES_MORE_THAN_ONE_LAST_RECORD_FLAG_WHOSE_ANCHOR_IS_UNDECIDED = 17,
}

/// CPU 恢复的一个失败在 GPU 结论里的失败点码（第 5、7 字）：`RecoveryFailure` 每个成员一个码，成员带的细节（盘、槽、哪一处）不进码。
/// 恢复没失败（没有文件、读到文件）的那一遍失败点是 [`GPU_FAILURE_POINT_NONE`]。
#[must_use]
pub fn gpu_failure_point_code(failure: &RecoveryFailure) -> u32 {
    match failure {
        RecoveryFailure::NoValidSystemConfiguration { .. } => {
            GPU_FAILURE_POINT_NO_VALID_SYSTEM_CONFIGURATION
        }
        RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized { .. } => {
            GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_INCOMPAT_BITS_NOT_RECOGNIZED
        }
        RecoveryFailure::SystemConfigurationsDisagree => {
            GPU_FAILURE_POINT_SYSTEM_CONFIGURATIONS_DISAGREE
        }
        RecoveryFailure::NoDeviceInThePool => GPU_FAILURE_POINT_NO_DEVICE_IN_THE_POOL,
        RecoveryFailure::RootRingSlotsPerRegionOutOfRange { .. } => {
            GPU_FAILURE_POINT_ROOT_RING_SLOTS_PER_REGION_OUT_OF_RANGE
        }
        RecoveryFailure::SystemConfigurationValueRefused { .. } => {
            GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_VALUE_REFUSED
        }
        RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart { .. } => {
            GPU_FAILURE_POINT_DEVICE_ENDS_BEFORE_THE_UNIT_AREA_START
        }
        RecoveryFailure::NoValidRoot => GPU_FAILURE_POINT_NO_VALID_ROOT,
        RecoveryFailure::UnitUnreadable { .. } => GPU_FAILURE_POINT_UNIT_UNREADABLE,
        RecoveryFailure::UnitMalformed { .. } => GPU_FAILURE_POINT_UNIT_MALFORMED,
        RecoveryFailure::InvariantViolated { .. } => GPU_FAILURE_POINT_INVARIANT_VIOLATED,
        RecoveryFailure::EntryNarrowerThanItsFieldTable { .. } => {
            GPU_FAILURE_POINT_ENTRY_NARROWER_THAN_ITS_FIELD_TABLE
        }
        RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree => {
            GPU_FAILURE_POINT_INODE_NUMBER_WATERMARK_ROW_MISSING_FROM_THE_ACCOUNTING_TREE
        }
        RecoveryFailure::AllocationRecordOutsideThePoolGeometry { .. } => {
            GPU_FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY
        }
        RecoveryFailure::MappingMiss { .. } => GPU_FAILURE_POINT_MAPPING_MISS,
        RecoveryFailure::MappingStillUnreadable { .. } => GPU_FAILURE_POINT_MAPPING_STILL_UNREADABLE,
        RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
            ..
        } => GPU_FAILURE_POINT_ROOT_PUBLISH_CARRIES_MORE_THAN_ONE_LAST_RECORD_FLAG_WHOSE_ANCHOR_IS_UNDECIDED,
    }
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

/// 一个状态在对拍里比的那几个字（判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 第三节 J6，用户 2026-10-10 定失败点这一版就比）：
/// 红项（两遍 oracle 与记录核对器的红位、池级 checker 的违例位）、两遍各自落到的根（实例代号、checkpoint_txg 低 32 位；没落到根时两个都是 `u32::MAX`）、
/// 两遍的走读结局与失败点（[`gpu_failure_point_code`]）、池级 checker 逐条判到位、GPU 判不了。GPU 那一侧由 [`cross_check_words_of_gpu_verdict`]、
/// CPU 那一侧由 [`cross_check_words_of_cpu_judgement`] 换出来，两份逐字段相等才算对上（CPU 那一侧判不了恒为假：GPU 判不了的状态对不上）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrossCheckWords {
    /// 结论第 0 字去掉判不了那一位：两遍 oracle 各七类（[`verdict_bits`]）、记录核对器两条。
    pub red_bits: u32,
    /// 池级 checker 的违例位（位 i ↔ `IMPLEMENTED_INVARIANTS[i]`）。
    pub pool_checker_violated_bits: u128,
    pub consulted_pass_landed_root: LandedRootWords,
    pub ignored_pass_landed_root: LandedRootWords,
    pub consulted_pass_outcome: u32,
    pub ignored_pass_outcome: u32,
    pub consulted_pass_failure_point: u32,
    pub ignored_pass_failure_point: u32,
    /// 池级 checker 逐条判到位（成立或违例）。
    pub pool_checker_judged_bits: u128,
    pub gpu_could_not_judge: bool,
}

/// 一遍恢复落到的根在结论里的两个字：实例代号、checkpoint_txg 低 32 位；没落到根时两个都是 `u32::MAX`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LandedRootWords {
    pub instance: u32,
    pub checkpoint_txg_low: u32,
}

impl LandedRootWords {
    /// 没落到根（择不到系统配置或根、重放在施加任何记录之前停下）。
    pub const NOT_LANDED: Self = Self {
        instance: u32::MAX,
        checkpoint_txg_low: u32::MAX,
    };
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
        gpu_could_not_judge: words[0] & undecidable_bit != 0
            || words[POOL_CHECKER_UNDECIDABLE_WORD] != 0,
    }
}

/// 一遍恢复的报告换成落到的根那两个字：`effective_root`（施加 journal 之后实际走的那条根；退回见证到的那一版时是退到的那一版）。
fn landed_root_words_of(
    effective_root: Option<(
        singlefs_core::address::InstanceGeneration,
        singlefs_core::address::CheckpointTxg,
    )>,
) -> LandedRootWords {
    effective_root.map_or(LandedRootWords::NOT_LANDED, |(instance, checkpoint_txg)| {
        LandedRootWords {
            instance: instance.0,
            checkpoint_txg_low: u32::try_from(checkpoint_txg.0 & 0xFFFF_FFFF).expect("低 32 位"),
        }
    })
}

/// 走读结局的码与失败点码（第 4 / 5 字或第 6 / 7 字）。
fn outcome_and_failure_point_of(outcome: &RecoveryOutcome) -> (u32, u32) {
    match outcome {
        RecoveryOutcome::NoFile { .. } => (RECOVERY_OUTCOME_CODE_NO_FILE, GPU_FAILURE_POINT_NONE),
        RecoveryOutcome::FileRead { .. } => {
            (RECOVERY_OUTCOME_CODE_FILE_READ, GPU_FAILURE_POINT_NONE)
        }
        RecoveryOutcome::Failed { failure, .. } => (
            RECOVERY_OUTCOME_CODE_FAILED,
            gpu_failure_point_code(failure),
        ),
    }
}

/// oracle 一类违例在红位里的位置（与 `Layer0OracleViolationKind::EVERY_KIND` 同序）。
fn oracle_kind_position(kind: Layer0OracleViolationKind) -> u32 {
    u32::try_from(
        Layer0OracleViolationKind::EVERY_KIND
            .iter()
            .position(|candidate| *candidate == kind)
            .expect("七类之一"),
    )
    .expect("七类，装得进 u32")
}

/// 池级 checker 一份判定的违例位与判到位（位 i ↔ `IMPLEMENTED_INVARIANTS[i]`；判到 = 成立或违例，不适用与部分没判都不算）。
///
/// # Panics
/// 判定里有一条不在 `IMPLEMENTED_INVARIANTS` 里。
#[must_use]
pub fn pool_checker_violated_and_judged_bits(
    verdicts: &[(&'static str, InvariantVerdict)],
) -> (u128, u128) {
    verdicts.iter().fold(
        (0u128, 0u128),
        |(violated, judged), (invariant, verdict)| {
            let bit = 1u128
                << IMPLEMENTED_INVARIANTS
                    .iter()
                    .position(|candidate| candidate == invariant)
                    .expect("池级 checker 报的不变量在清单里");
            match verdict {
                InvariantVerdict::Violated(_) => (violated | bit, judged | bit),
                InvariantVerdict::Holds => (violated, judged | bit),
                InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => {
                    (violated, judged)
                }
            }
        },
    )
}

/// CPU 判器一个状态的判定换成对拍比的那几个字（池级 checker 取判定里那一份；旋钮值编进内核的对拍用例拿带同一组旋钮值的入口重判之后
/// 自己换掉违例位与判到位）。
#[must_use]
pub fn cross_check_words_of_cpu_judgement(judgement: &Layer0StateJudgement) -> CrossCheckWords {
    let mut red_bits = 0u32;
    if let Some(violation) = &judgement.consulted_oracle {
        red_bits |=
            1 << (verdict_bits::CONSULTED_ORACLE_FIRST + oracle_kind_position(violation.kind));
    }
    if let Some(violation) = &judgement.ignored_oracle {
        red_bits |=
            1 << (verdict_bits::IGNORED_ORACLE_FIRST + oracle_kind_position(violation.kind));
    }
    if judgement.records.root_without_record {
        red_bits |= 1 << verdict_bits::ROOT_WITHOUT_RECORD;
    }
    if judgement.records.claimed_state_missing_unit {
        red_bits |= 1 << verdict_bits::CLAIMED_STATE_MISSING_UNIT;
    }
    let (pool_checker_violated_bits, pool_checker_judged_bits) =
        pool_checker_violated_and_judged_bits(&judgement.pool_checker);
    let (consulted_pass_outcome, consulted_pass_failure_point) =
        outcome_and_failure_point_of(&judgement.consulted.outcome);
    let (ignored_pass_outcome, ignored_pass_failure_point) =
        outcome_and_failure_point_of(&judgement.ignored.outcome);
    CrossCheckWords {
        red_bits,
        pool_checker_violated_bits,
        consulted_pass_landed_root: landed_root_words_of(judgement.consulted.effective_root),
        ignored_pass_landed_root: landed_root_words_of(judgement.ignored.effective_root),
        consulted_pass_outcome,
        ignored_pass_outcome,
        consulted_pass_failure_point,
        ignored_pass_failure_point,
        pool_checker_judged_bits,
        gpu_could_not_judge: false,
    }
}
/// 每个状态的草稿区几个 u32（checker 共用那一半里 `SCRATCH_END`：共用的 `SCRATCH_WORDS_REQUIRED`、走读与扫描的集合、两个内核的交接段、
/// 交接段之后走读内核自己用的表：两张，加 I-3.5 的出现规则读的四张；再之后是扫描内核删除态扫描与墓碑、谱系那一遍用的五张：inode 记录、每个号的数据单元数、
/// 一棵墓碑树或谱系树的容器、一版的谱系记录、I-9.29 跨版本的号 → 对象出生代，与 I-9.30 读的最新根记账里 inode 号水位那一统计量的行，
/// 再加退役那几条读的两张叶身份表：被判的那一版与 I-9.35 的父版；最后是判到位的两张表：评估过、射程里有一部分判不了，各 4 个字）。
pub const SCRATCH_WORDS: usize = 320_696;
/// 内核的工作组大小（三个内核都是 `@workgroup_size(64)`）：派 n 个状态要 ⌈n ÷ 64⌉ 个工作组。
pub const WORKGROUP_SIZE: usize = 64;
/// 内核的四个绑定（`crash_judge_common.wgsl` 开头）：派活参数、打包的表、结论、草稿区。
pub const DISPATCH_BINDING: u32 = 0;
pub const TABLES_BINDING: u32 = 1;
pub const VERDICTS_BINDING: u32 = 2;
pub const SCRATCH_BINDING: u32 = 3;
/// 派活那一侧（`crate::crash_judge_dispatch`，不在判法闭包里）修了会改结论的毛病时抬这个数：
/// 判法版本跟着变，库里旧行留作历史，这一版从头判。只改快慢、不改结论的改动不抬。
pub const GPU_JUDGE_REVISION: u32 = 1;

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

/// 这个状态的结论是不是两个内核都写过（不是初值）。
#[must_use]
pub fn verdict_was_written_by_both_kernels(words: &[u32; VERDICT_WORDS]) -> bool {
    (words[0] != VERDICT_WORD_NOT_WRITTEN || words[1] != VERDICT_WORD_NOT_WRITTEN)
        && words[POOL_CHECKER_REACHED_WORD] != VERDICT_WORD_NOT_WRITTEN
}

/// 红位。
pub mod verdict_bits {
    /// 看 journal 那一遍 oracle 的七类从这一位起（与 `Layer0OracleViolationKind::EVERY_KIND` 同序）。
    pub const CONSULTED_ORACLE_FIRST: u32 = 0;
    /// 不看 journal 那一遍从这一位起。
    pub const IGNORED_ORACLE_FIRST: u32 = 8;
    pub const ROOT_WITHOUT_RECORD: u32 = 16;
    pub const CLAIMED_STATE_MISSING_UNIT: u32 = 17;
    pub const UNDECIDABLE: u32 = 31;
}

/// 判法代码的摘要：内核前面那一份生成的常量（[`kernel_prelude_wgsl_constants`]）、三棵容器树都没装旋钮的那一份旋钮常量与五份内核源码，
/// 连同一个版本前缀。
#[must_use]
pub fn gpu_judge_digest() -> [u8; DIGEST_BYTES] {
    let mut message = b"gpu-crash-judge-v6\n".to_vec();
    message.extend_from_slice(kernel_prelude_wgsl_constants().as_bytes());
    message.extend_from_slice(
        container_capacity_knob_wgsl_constants(ContainerCapacityKnobs::PRODUCT).as_bytes(),
    );
    message.extend_from_slice(GPU_JUDGE_COMMON_SOURCE.as_bytes());
    message.extend_from_slice(GPU_JUDGE_RECOVERY_SOURCE.as_bytes());
    message.extend_from_slice(GPU_JUDGE_CHECKER_COMMON_SOURCE.as_bytes());
    message.extend_from_slice(GPU_JUDGE_CHECKER_SOURCE.as_bytes());
    message.extend_from_slice(GPU_JUDGE_CHECKER_SCAN_SOURCE.as_bytes());
    sha256_digest(&message)
}

/// 恢复内核的整份源码（生成的常量、共用那一半接上它自己那一半）。
#[must_use]
pub fn recovery_kernel_source() -> String {
    format!(
        "{}\n{GPU_JUDGE_COMMON_SOURCE}\n{GPU_JUDGE_RECOVERY_SOURCE}",
        kernel_prelude_wgsl_constants()
    )
}

/// 序号 → 持久掩码那一段的探针（对拍用例用，不判状态）：每个状态 [`PERSISTED_MASK_PROBE_WORDS`] 个字，前 64 个是共用那一半 `begin_state`
/// 拆出的持久掩码（位 i 在第 i / 32 字的第 i % 32 位），最后一个是它判不了的原因（0 是判得了）。
pub const PERSISTED_MASK_PROBE_WORDS: usize = 65;

const PERSISTED_MASK_PROBE_ENTRY_SOURCE: &str = r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let local = id.x;
    if (local >= dispatch.state_count) {
        return;
    }
    begin_state(local);
    let out = local * PERSISTED_MASK_PROBE_WORDS;
    for (var word = 0u; word < 64u; word = word + 1u) {
        verdicts[out + word] = persisted[word];
    }
    verdicts[out + 64u] = undecidable;
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
/// `words` 不是 [`PERSISTED_MASK_PROBE_WORDS`] 个字；`enumerated_write_count` 超过掩码的宽（2048）。
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
        enumerated_write_count <= 64 * 32,
        "枚举写表装得进 64 个字的掩码"
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
/// 三个都为 0 的那一份（[`ContainerCapacityKnobs::PRODUCT`]）进判法摘要。
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

/// 池级 checker 扫描内核的整份源码（三棵容器树都没装旋钮：层 0 与放量编的那一份）。
#[must_use]
pub fn checker_scan_kernel_source() -> String {
    checker_scan_kernel_source_given_the_container_capacity_knobs(ContainerCapacityKnobs::PRODUCT)
}

/// 池级 checker 扫描内核的整份源码，三棵容器树的旋钮值编进内核（[`container_capacity_knob_wgsl_constants`]）。
#[must_use]
pub fn checker_scan_kernel_source_given_the_container_capacity_knobs(
    knobs: ContainerCapacityKnobs,
) -> String {
    format!(
        "{}\n{}\n{GPU_JUDGE_COMMON_SOURCE}\n{GPU_JUDGE_CHECKER_COMMON_SOURCE}\n{GPU_JUDGE_CHECKER_SCAN_SOURCE}",
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
    /// 判法摘要 [`gpu_judge_digest`] 罩住的那三份。
    #[must_use]
    pub fn of_the_judge() -> Self {
        Self::of_the_judge_given_the_container_capacity_knobs(ContainerCapacityKnobs::PRODUCT)
    }

    /// 同 [`Self::of_the_judge`]，三棵容器树的旋钮值编进扫描内核（第 56 批 G6）：判出的池级 checker 违例位要与
    /// `singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs` 带同一组旋钮值逐位相同。旋钮值不进判法摘要：
    /// 只给对拍用例判旋钮压小写出的镜像，判定入库的路径编 [`Self::of_the_judge`]。
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

    /// GPU 结论字换成对拍字：红位去掉判不了那一位、两遍的根与结局与失败点各取各的字、违例位与判到位各读四个字；判不了两种来路都认。
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
}
