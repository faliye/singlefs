//! GPU 判器（用户 2026-09-28 定「GPU 判器现在实现、整套做完」）：把 `crate::crash_judge_tables` 摊出来的表整份上卡，
//! 每个调用判一个崩溃状态——恢复（看 journal 与不看 journal 各一遍：择系统配置、择根、扫环重放、走到第一个文件）、
//! 七类 oracle、记录核对器两条判据，判法与 CPU 判器（`crate::crash::judge_crash_image`）逐条相同；GPU 与 CPU 逐状态比是它的判别力自证。
//! 内核分五份源码：`crash_judge_common.wgsl`（各内核共用的解析与查表）、`crash_judge_recovery.wgsl`（恢复、oracle、记录核对器）、
//! `crash_judge_checker_common.wgsl`（池级 checker 两个内核共用的读法与交接段）、`crash_judge_checker.wgsl`（走读内核：第一、二批不变量，
//! 连同里程碑三二·1 那一批里的十五条与 I-9.27）、`crash_judge_checker_scan.wgsl`（扫描内核：第三批、I-3.12 与 I-5.5）；判不了的状态（表越过内核容量）置最高位、交 CPU 判。
//!
//! 一个状态的结论 [`VERDICT_WORDS`] 个 u32：第 0 字是红位（位 0..6 看 journal 那一遍 oracle 的七类、位 8..14 不看那一遍、
//! 位 16 根在而记录都不在、位 17 恢复自称的状态少单元、位 31 判不了）；第 1 字判不了的原因（判得了时低 16 位是看 journal 那一遍
//! 重放停在哪一道、高 16 位是记录数，诊断用）；第 2 / 3 字看 journal 那一遍落到的
//! 实例代号与 checkpoint_txg 低 32 位；第 4 / 5 字它的走读结局与失败点；第 6 / 7 字不看那一遍的；
//! 第 8..12 字池级 checker 的违例位（[`POOL_CHECKER_VIOLATED_WORDS`] 个字，128 位）；第 12 字池级 checker 走到第几步；第 13 字它判不了的原因。
//! 这几个字号在内核里的常量由 [`verdict_layout_wgsl_constants`] 从这里生成、接在每个内核源码前面，不在 WGSL 里另写一份。
//! 判法代码的摘要 [`gpu_judge_digest`] 进判定表的键：内核改一个字，旧行留着当历史，新行另起。
//!
//! 这一份只放判法：内核原文、表怎么打包、派活参数与结论各是哪几个字、草稿区每个状态多大。怎么上卡、一次派活带几个状态、
//! 几张卡怎么用，在 `crate::crash_judge_dispatch`——那一份不在判法闭包里（用户 2026-09-29 定「派活和判法拆出来最好」），
//! 改它不换判法版本；这一份不许引它。
use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};

use crate::crash_judge_tables::JudgeTablesWords;

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
/// 每个状态的结论几个 u32：恢复内核 8 个，池级 checker 内核违例位、走到第几步、判不了的原因。
pub const VERDICT_WORDS: usize = POOL_CHECKER_UNDECIDABLE_WORD + 1;

// 违例位装得下池级 checker 的每一条不变量；读出来的是一个 u128，字数再多就装不进。
const _: () = assert!(
    singlefs_checker::image::IMPLEMENTED_INVARIANTS.len() <= POOL_CHECKER_VIOLATED_BIT_CAPACITY,
    "池级 checker 的不变量多于违例位的容量：先加宽 POOL_CHECKER_VIOLATED_WORDS"
);
const _: () = assert!(
    POOL_CHECKER_VIOLATED_BIT_CAPACITY <= 128,
    "违例位读成一个 u128"
);

/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：前 65 条——三批 49 条，加里程碑三二·1 那一批 16 条
/// （判决 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行），再加 C673（三 oracle 合谋测试的补 oracle 改法没做）
/// ⑤ ⑥ 那两条（下标 66 是 I-9.27，走读内核挨着 I-9.16 按表判；下标 67 是 I-5.5，扫描内核每一版重读树表与 extent 树判），对拍逐位比、不屏蔽。
pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 66, 67,
];

/// 池级 checker 已经判、GPU 内核还没判的不变量：下标 65 是 I-3.5（引用区间的精确性，里程碑三第 8 批进池级 checker，
/// 派发没含 GPU 内核那一半，交主 agent 定）。内核那一位恒 0：对拍逐位比照旧不屏蔽，CPU 在哪个状态上判红它，对拍就在那个状态上报不同；
/// 崩溃放量由 GPU 判的那几趟判不到它（`crash_amplification` 按违例位取不变量）。
pub const POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET: &[usize] = &[65];

// 覆盖表与「内核还没判」那张表合起来是池级 checker 的每一条：每个下标恰好在其中一张表里、两张表各按下标升序。
// 新立一条不变量而两张表都没跟上时这里编译不过，不悄悄让 GPU 那一位恒 0；还没判的那几条要在「内核还没判」那张表里点名。
// 「内核还没判」那几条不必在覆盖表的末尾：I-3.5（下标 65）之后立的 I-9.27、I-5.5 内核判了，排在它后面。
const _: () = {
    assert!(
        POOL_CHECKER_COVERED_INVARIANT_INDEXES.len()
            + POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET.len()
            == singlefs_checker::image::IMPLEMENTED_INVARIANTS.len(),
        "池级 checker 新立的不变量 GPU 内核还没判：补内核，再把下标补进覆盖表（暂不补内核的，点名进「内核还没判」那张表）"
    );
    let mut position = 1;
    while position < POOL_CHECKER_COVERED_INVARIANT_INDEXES.len() {
        assert!(
            POOL_CHECKER_COVERED_INVARIANT_INDEXES[position - 1]
                < POOL_CHECKER_COVERED_INVARIANT_INDEXES[position],
            "覆盖表按下标升序"
        );
        position += 1;
    }
    let mut position_not_judged = 1;
    while position_not_judged < POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET.len() {
        assert!(
            POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET[position_not_judged - 1]
                < POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET[position_not_judged],
            "「内核还没判」那张表按下标升序"
        );
        position_not_judged += 1;
    }
    // 每个下标恰好在一张表里：两张表都升序、合起来条数等于不变量条数，再要求每个下标至少在一张表里，就一条不缺、一条不重。
    let mut invariant_index = 0;
    while invariant_index < singlefs_checker::image::IMPLEMENTED_INVARIANTS.len() {
        let mut is_listed = false;
        let mut covered_position = 0;
        while covered_position < POOL_CHECKER_COVERED_INVARIANT_INDEXES.len() {
            if POOL_CHECKER_COVERED_INVARIANT_INDEXES[covered_position] == invariant_index {
                is_listed = true;
            }
            covered_position += 1;
        }
        let mut not_judged_position = 0;
        while not_judged_position
            < POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET.len()
        {
            if POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET[not_judged_position]
                == invariant_index
            {
                is_listed = true;
            }
            not_judged_position += 1;
        }
        assert!(
            is_listed,
            "池级 checker 的每一条要么在覆盖表里、要么在「内核还没判」那张表里"
        );
        invariant_index += 1;
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

/// 结论的字号在内核里的常量（WGSL 源码）：接在每个内核源码最前面，判法摘要也收它。
#[must_use]
pub fn verdict_layout_wgsl_constants() -> String {
    format!(
        "const VERDICT_WORDS: u32 = {VERDICT_WORDS}u;\n\
         const POOL_CHECKER_VIOLATED_FIRST_WORD: u32 = {POOL_CHECKER_VIOLATED_FIRST_WORD}u;\n\
         const POOL_CHECKER_VIOLATED_WORDS: u32 = {POOL_CHECKER_VIOLATED_WORDS}u;\n\
         const POOL_CHECKER_REACHED_WORD: u32 = {POOL_CHECKER_REACHED_WORD}u;\n\
         const POOL_CHECKER_UNDECIDABLE_WORD: u32 = {POOL_CHECKER_UNDECIDABLE_WORD}u;\n"
    )
}
/// 每个状态的草稿区几个 u32（checker 共用那一半里 `SCRATCH_END`：共用的 `SCRATCH_WORDS_REQUIRED`、走读与扫描的集合、两个内核的交接段、
/// 交接段之后走读内核自己用的两张表）。
pub const SCRATCH_WORDS: usize = 255_504;
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

/// 判法代码的摘要：结论字号的常量与五份内核源码，连同一个版本前缀。
#[must_use]
pub fn gpu_judge_digest() -> [u8; DIGEST_BYTES] {
    let mut message = b"gpu-crash-judge-v5\n".to_vec();
    message.extend_from_slice(verdict_layout_wgsl_constants().as_bytes());
    message.extend_from_slice(GPU_JUDGE_COMMON_SOURCE.as_bytes());
    message.extend_from_slice(GPU_JUDGE_RECOVERY_SOURCE.as_bytes());
    message.extend_from_slice(GPU_JUDGE_CHECKER_COMMON_SOURCE.as_bytes());
    message.extend_from_slice(GPU_JUDGE_CHECKER_SOURCE.as_bytes());
    message.extend_from_slice(GPU_JUDGE_CHECKER_SCAN_SOURCE.as_bytes());
    sha256_digest(&message)
}

/// 恢复内核的整份源码（结论字号的常量、共用那一半接上它自己那一半）。
#[must_use]
pub fn recovery_kernel_source() -> String {
    format!(
        "{}\n{GPU_JUDGE_COMMON_SOURCE}\n{GPU_JUDGE_RECOVERY_SOURCE}",
        verdict_layout_wgsl_constants()
    )
}

/// 池级 checker 内核的整份源码。
#[must_use]
pub fn checker_walk_kernel_source() -> String {
    format!(
        "{}\n{GPU_JUDGE_COMMON_SOURCE}\n{GPU_JUDGE_CHECKER_COMMON_SOURCE}\n{GPU_JUDGE_CHECKER_SOURCE}",
        verdict_layout_wgsl_constants()
    )
}

/// 池级 checker 扫描内核的整份源码。
#[must_use]
pub fn checker_scan_kernel_source() -> String {
    format!(
        "{}\n{GPU_JUDGE_COMMON_SOURCE}\n{GPU_JUDGE_CHECKER_COMMON_SOURCE}\n{GPU_JUDGE_CHECKER_SCAN_SOURCE}",
        verdict_layout_wgsl_constants()
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
        Self {
            recovery: recovery_kernel_source(),
            checker_walk: checker_walk_kernel_source(),
            checker_scan: checker_scan_kernel_source(),
        }
    }
}

/// 全部表按目录打包成一张 u32 表：前 38 个字是目录（每张表的起点、条数），之后依次是各表。
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
