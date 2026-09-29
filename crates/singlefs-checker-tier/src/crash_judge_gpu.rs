//! GPU 判器（用户 2026-09-28 定「GPU 判器现在实现、整套做完」）：把 `crate::crash_judge_tables` 摊出来的表整份上卡，
//! 每个调用判一个崩溃状态——恢复（看 journal 与不看 journal 各一遍：择系统配置、择根、扫环重放、走到第一个文件）、
//! 七类 oracle、记录核对器两条判据，判法与 CPU 判器（`crate::crash::judge_crash_image`）逐条相同；GPU 与 CPU 逐状态比是它的判别力自证。
//! 内核分五份源码：`crash_judge_common.wgsl`（各内核共用的解析与查表）、`crash_judge_recovery.wgsl`（恢复、oracle、记录核对器）、
//! `crash_judge_checker_common.wgsl`（池级 checker 两个内核共用的读法与交接段）、`crash_judge_checker.wgsl`（走读内核：第一、二批不变量）、
//! `crash_judge_checker_scan.wgsl`（扫描内核：第三批）；判不了的状态（表越过内核容量）置最高位、交 CPU 判。
//!
//! 一个状态的结论 [`VERDICT_WORDS`] 个 u32：第 0 字是红位（位 0..6 看 journal 那一遍 oracle 的七类、位 8..14 不看那一遍、
//! 位 16 根在而记录都不在、位 17 恢复自称的状态少单元、位 31 判不了）；第 1 字判不了的原因（判得了时低 16 位是看 journal 那一遍
//! 重放停在哪一道、高 16 位是记录数，诊断用）；第 2 / 3 字看 journal 那一遍落到的
//! 实例代号与 checkpoint_txg 低 32 位；第 4 / 5 字它的走读结局与失败点；第 6 / 7 字不看那一遍的。
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
/// 池级 checker 走读内核：第一、二批不变量的违例位（写第 8..12 字），走完把计数与几何交进草稿区的交接段。
pub const GPU_JUDGE_CHECKER_SOURCE: &str = include_str!("crash_judge_checker.wgsl");
/// 池级 checker 扫描内核：第三批（扫描方向与记账那九条），违例位并进第 8、9 字。
pub const GPU_JUDGE_CHECKER_SCAN_SOURCE: &str = include_str!("crash_judge_checker_scan.wgsl");

/// 每个状态的结论几个 u32：恢复内核 8 个，池级 checker 内核 4 个。
pub const VERDICT_WORDS: usize = 12;
/// 池级 checker 违例位的低 32 位住第几字（位 i ↔ `singlefs_checker::image::IMPLEMENTED_INVARIANTS[i]`），高位在下一字。
pub const POOL_CHECKER_VIOLATED_LOW_WORD: usize = 8;
/// 池级 checker 内核在这个状态上走到了第几步（0 盘数超上限、1 有槽带拒收的值、2 择到了系统配置、3 根环非空并判完第一批、4 走读那一批判完、5 扫描那一批判完）。
pub const POOL_CHECKER_REACHED_WORD: usize = 10;
/// 池级 checker 内核判不了的原因。
pub const POOL_CHECKER_UNDECIDABLE_WORD: usize = 11;

/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：三批接完之后是全部 49 条，对拍逐位比。
pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48,
];

/// 一个状态结论里池级 checker 的违例位（64 位，位 i ↔ `IMPLEMENTED_INVARIANTS[i]`）。
#[must_use]
pub fn pool_checker_violated_bits(words: &[u32; VERDICT_WORDS]) -> u64 {
    u64::from(words[POOL_CHECKER_VIOLATED_LOW_WORD])
        | (u64::from(words[POOL_CHECKER_VIOLATED_LOW_WORD + 1]) << 32)
}
/// 每个状态的草稿区几个 u32（checker 共用那一半里 `H_END`：共用的 `SCRATCH_WORDS_REQUIRED`、走读与扫描的集合、两个内核的交接段）。
pub const SCRATCH_WORDS: usize = 247_312;
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

/// 判法代码的摘要：五份内核源码连同一个版本前缀。
#[must_use]
pub fn gpu_judge_digest() -> [u8; DIGEST_BYTES] {
    let mut message = b"gpu-crash-judge-v3\n".to_vec();
    message.extend_from_slice(GPU_JUDGE_COMMON_SOURCE.as_bytes());
    message.extend_from_slice(GPU_JUDGE_RECOVERY_SOURCE.as_bytes());
    message.extend_from_slice(GPU_JUDGE_CHECKER_COMMON_SOURCE.as_bytes());
    message.extend_from_slice(GPU_JUDGE_CHECKER_SOURCE.as_bytes());
    message.extend_from_slice(GPU_JUDGE_CHECKER_SCAN_SOURCE.as_bytes());
    sha256_digest(&message)
}

/// 恢复内核的整份源码（共用那一半接上它自己那一半）。
#[must_use]
pub fn recovery_kernel_source() -> String {
    format!("{GPU_JUDGE_COMMON_SOURCE}\n{GPU_JUDGE_RECOVERY_SOURCE}")
}

/// 池级 checker 内核的整份源码。
#[must_use]
pub fn checker_walk_kernel_source() -> String {
    format!(
        "{GPU_JUDGE_COMMON_SOURCE}\n{GPU_JUDGE_CHECKER_COMMON_SOURCE}\n{GPU_JUDGE_CHECKER_SOURCE}"
    )
}

/// 池级 checker 扫描内核的整份源码。
#[must_use]
pub fn checker_scan_kernel_source() -> String {
    format!("{GPU_JUDGE_COMMON_SOURCE}\n{GPU_JUDGE_CHECKER_COMMON_SOURCE}\n{GPU_JUDGE_CHECKER_SCAN_SOURCE}")
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
        recovery_kernel_source, verdict_was_written_by_both_kernels, SCRATCH_WORDS, VERDICT_WORDS,
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
