//! 扫描里的校验和核对（D26（后台整理与放置回收） 已定项 13 第 1 条点名的三段里的「甲」）：两份独立写的 WGSL 核（第 8 条，不共用核函数），
//! 各自对同一批单元字节算 CRC-32C 并与期望值比。这里是两份核共用的缓冲布局、打包与回读解码，都是纯函数；起 GPU 的在 `background_gpu`。
//!
//! 缓冲布局（两份核按同一个绑定号读写，各有各的输出缓冲）：
//! - 绑定 0，uniform：这一段的条数（一个 `u32`，缓冲占 [`PARAMETER_BUFFER_BYTES`]）。
//! - 绑定 1，只读存储：这一段每条的单元字节首尾相接，不对齐、不留空，末尾补 0 到 4 字节的整数倍；着色器按小端 `u32` 读。
//! - 绑定 2，只读存储：每条一项描述，三个 `u32`：在输入里的起点字节、字节数、期望的 CRC-32C。
//! - 绑定 3，读写存储：每条一项结果，两个 `u32`：算出的 CRC-32C、判定字（1 是与期望相同，0 是不同）。
//!   开算之前整块填 [`OUTCOME_NOT_WRITTEN_FILL_WORD`]：核没写到的那一条读回来判定字不是 0 也不是 1，按 GPU 出错交回。

use crate::memory_limit_and_chunks::ChunkFootprint;
use crate::two_kernel_reconciliation::{
    ConsecutiveUncertaintyCounts, ReconciledBatch, TwoKernelOutcomesOfAnEntry,
};

/// 按字节查表的那一份核（`crc32c_by_byte_remainder_table`）。
pub const TABLE_DRIVEN_KERNEL_SOURCE: &str = include_str!("checksum_kernel_table_driven.wgsl");
pub const TABLE_DRIVEN_KERNEL_ENTRY_POINT: &str = "crc32c_by_byte_remainder_table";
/// 按 32 位字逐位移算的那一份核（`crc32c_by_shifting_whole_words`）。
pub const BITWISE_KERNEL_SOURCE: &str = include_str!("checksum_kernel_bitwise.wgsl");
pub const BITWISE_KERNEL_ENTRY_POINT: &str = "crc32c_by_shifting_whole_words";

/// 两份核的工作组都是 64 个调用（两份 WGSL 里的 `@workgroup_size(64)`），每条一个调用。
pub const INVOCATIONS_PER_WORKGROUP: u32 = 64;
/// 参数缓冲占的字节数：着色器只读开头一个 `u32`，缓冲按 16 字节建（uniform 缓冲的常见最小对齐）。
pub const PARAMETER_BUFFER_BYTES: u64 = 16;
/// 每条描述三个 `u32`。
pub const DESCRIPTOR_WORDS_PER_ENTRY: usize = 3;
pub const DESCRIPTOR_BYTES_PER_ENTRY: u64 = 12;
/// 每条结果两个 `u32`。
pub const OUTCOME_WORDS_PER_ENTRY: usize = 2;
pub const OUTCOME_BYTES_PER_ENTRY: u64 = 8;
/// 结果缓冲开算之前的填充字：判定字只会被核写成 0 或 1。
pub const OUTCOME_NOT_WRITTEN_FILL_WORD: u32 = 0xFFFF_FFFF;
const VERDICT_WORD_MATCHES: u32 = 1;
const VERDICT_WORD_DOES_NOT_MATCH: u32 = 0;
/// 两份核各一个结果缓冲与一个回读暂存缓冲。
const OUTCOME_AND_READBACK_BUFFERS: u64 = 4;

/// 校验和核对一段在卡上占的：参数缓冲；每条一项描述、两份核各一项结果与一项回读暂存。按条排的最大缓冲是描述那一个。
pub const CHECKSUM_CHUNK_FOOTPRINT: ChunkFootprint = ChunkFootprint {
    bytes_per_chunk_besides_the_entries: PARAMETER_BUFFER_BYTES,
    bytes_per_entry_besides_its_input: DESCRIPTOR_BYTES_PER_ENTRY
        + OUTCOME_AND_READBACK_BUFFERS * OUTCOME_BYTES_PER_ENTRY,
    bytes_per_entry_in_the_largest_entry_buffer: DESCRIPTOR_BYTES_PER_ENTRY,
};

/// 要核的一个单元：调用方给的条目键（同一单元在下一轮还用同一个键，连续不确定按它数）、单元字节、期望的 CRC-32C。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitChecksumToVerify<'unit, EntryKey> {
    pub entry_key: EntryKey,
    pub unit_bytes: &'unit [u8],
    pub expected_crc32c: u32,
}

/// 一份核对一条交回的结果。两份核逐条比的就是它（两样都比：算出的值不同、判定相同也算不确定）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChecksumKernelOutcome {
    pub computed_crc32c: u32,
    pub matches_the_expected_crc32c: bool,
}

/// 打包好的一段：输入字节（已补到 4 字节的整数倍）与描述的字。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedChecksumChunk {
    pub input_bytes: Vec<u8>,
    pub descriptor_words: Vec<u32>,
}

impl PackedChecksumChunk {
    #[must_use]
    pub fn entry_count(&self) -> usize {
        self.descriptor_words.len() / DESCRIPTOR_WORDS_PER_ENTRY
    }

    /// 描述按小端字节交给缓冲。
    #[must_use]
    pub fn descriptor_bytes(&self) -> Vec<u8> {
        self.descriptor_words
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect()
    }
}

/// 把一段单元按次序首尾相接打包：每个单元的起点就是前面那些单元字节数之和，不按字对齐（着色器按字节取）。
///
/// # Panics
/// 一段的输入超过 `u32` 能表示的字节数：切批按绑定上限切过（外壳把绑定上限压在 `u32` 之内）。
#[must_use]
pub fn packed_checksum_chunk<EntryKey>(
    units: &[UnitChecksumToVerify<'_, EntryKey>],
) -> PackedChecksumChunk {
    let total_unit_bytes: usize = units.iter().map(|unit| unit.unit_bytes.len()).sum();
    let padded_bytes = crate::memory_limit_and_chunks::input_buffer_bytes(
        u64::try_from(total_unit_bytes).expect("usize 装得进 u64"),
    );
    let mut input_bytes =
        Vec::with_capacity(usize::try_from(padded_bytes).expect("一段的输入装得进内存"));
    let mut descriptor_words = Vec::with_capacity(units.len() * DESCRIPTOR_WORDS_PER_ENTRY);
    for unit in units {
        let byte_offset = u32::try_from(input_bytes.len())
            .expect("一段的输入不超过 u32 字节：切批按绑定上限切过");
        let byte_length = u32::try_from(unit.unit_bytes.len())
            .expect("一个单元不超过 u32 字节：切批按绑定上限切过");
        input_bytes.extend_from_slice(unit.unit_bytes);
        descriptor_words.extend_from_slice(&[byte_offset, byte_length, unit.expected_crc32c]);
    }
    input_bytes.resize(
        usize::try_from(padded_bytes).expect("一段的输入装得进内存"),
        0,
    );
    PackedChecksumChunk {
        input_bytes,
        descriptor_words,
    }
}

/// 结果缓冲开算之前的内容：每个字都是填充字。
#[must_use]
pub fn outcome_buffer_before_the_kernel_writes(entry_count: usize) -> Vec<u8> {
    std::iter::repeat_n(
        OUTCOME_NOT_WRITTEN_FILL_WORD.to_le_bytes(),
        entry_count * OUTCOME_WORDS_PER_ENTRY,
    )
    .flatten()
    .collect()
}

/// 读回的一条结果判定字不是 0 也不是 1：核没写到这一条（还是填充字）或写坏了，这一批按 GPU 出错交回。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnrecognizedVerdictWord {
    pub entry_position: usize,
    pub verdict_word: u32,
}

/// 把读回的结果字节解成每条一个结果。
///
/// # Errors
/// 某一条的判定字认不得（[`UnrecognizedVerdictWord`]）。
///
/// # Panics
/// 字节数不是每条 8 字节的整数倍：回读缓冲按条数建，长度对不上是外壳的 bug。
pub fn decoded_kernel_outcomes(
    readback_bytes: &[u8],
) -> Result<Vec<ChecksumKernelOutcome>, UnrecognizedVerdictWord> {
    let (outcome_records, leftover_bytes) = readback_bytes.as_chunks::<8>();
    assert!(
        leftover_bytes.is_empty(),
        "回读的字节数是每条 8 字节的整数倍"
    );
    outcome_records
        .iter()
        .enumerate()
        .map(|(entry_position, outcome_record)| {
            let computed_crc32c = u32::from_le_bytes([
                outcome_record[0],
                outcome_record[1],
                outcome_record[2],
                outcome_record[3],
            ]);
            let verdict_word = u32::from_le_bytes([
                outcome_record[4],
                outcome_record[5],
                outcome_record[6],
                outcome_record[7],
            ]);
            let matches_the_expected_crc32c = match verdict_word {
                VERDICT_WORD_MATCHES => true,
                VERDICT_WORD_DOES_NOT_MATCH => false,
                unrecognized_verdict_word => {
                    return Err(UnrecognizedVerdictWord {
                        entry_position,
                        verdict_word: unrecognized_verdict_word,
                    })
                }
            };
            Ok(ChecksumKernelOutcome {
                computed_crc32c,
                matches_the_expected_crc32c,
            })
        })
        .collect()
}

/// CPU 那一份实现对一条的判定（连续不确定的条目交它判；对拍也拿它当参照）：核心层的 CRC-32C。
#[must_use]
pub fn cpu_reference_outcome(unit_bytes: &[u8], expected_crc32c: u32) -> ChecksumKernelOutcome {
    let computed_crc32c = singlefs_core::checksum::crc32_castagnoli(unit_bytes);
    ChecksumKernelOutcome {
        computed_crc32c,
        matches_the_expected_crc32c: computed_crc32c == expected_crc32c,
    }
}

/// 扫描里的校验和核对一批的比对（第 8 条）：两份核的结果交进来（按 `units` 的次序，查表那一份在前），逐条比、更新计数；
/// 连续不确定到 N 轮的条目交 CPU 参照（[`cpu_reference_outcome`]）现判。这一批出错就原样交回出错、计数不动、CPU 参照不调。
///
/// # Errors
/// `both_kernel_outcomes` 是 `Err` 时交回它。
///
/// # Panics
/// 两份核交回的条数与 `units` 不等；同一个条目键在一批里出现两次。
pub fn reconciled_unit_checksums<EntryKey: Eq + std::hash::Hash + Clone, BatchFailure>(
    units: &[UnitChecksumToVerify<'_, EntryKey>],
    both_kernel_outcomes: Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, BatchFailure>,
    uncertainty: &mut ConsecutiveUncertaintyCounts<EntryKey>,
) -> Result<ReconciledBatch<ChecksumKernelOutcome>, BatchFailure> {
    let both_kernel_outcomes = both_kernel_outcomes.map(|outcomes| {
        assert_eq!(outcomes.len(), units.len(), "两份核每条各交一个结果");
        units
            .iter()
            .zip(outcomes)
            .map(
                |(unit, (table_driven, bitwise))| TwoKernelOutcomesOfAnEntry {
                    entry_key: unit.entry_key.clone(),
                    outcome_by_one_kernel: table_driven,
                    outcome_by_the_independently_written_kernel: bitwise,
                },
            )
            .collect()
    });
    uncertainty.reconcile_unless_the_batch_failed(both_kernel_outcomes, |entry_position| {
        cpu_reference_outcome(
            units[entry_position].unit_bytes,
            units[entry_position].expected_crc32c,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(unit_bytes: &[u8], expected_crc32c: u32) -> UnitChecksumToVerify<'_, u32> {
        UnitChecksumToVerify {
            entry_key: expected_crc32c,
            unit_bytes,
            expected_crc32c,
        }
    }

    #[test]
    fn units_are_packed_end_to_end_at_unaligned_offsets_and_padded_to_a_whole_word() {
        let packed = packed_checksum_chunk(&[
            unit(&[0xA1], 11),
            unit(&[0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7], 22),
            unit(&[], 33),
            unit(&[0xC1, 0xC2, 0xC3], 44),
        ]);
        assert_eq!(
            packed.input_bytes,
            vec![0xA1, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xC1, 0xC2, 0xC3, 0x00],
            "首尾相接，末尾补到 12 字节"
        );
        assert_eq!(
            packed.descriptor_words,
            vec![0, 1, 11, 1, 7, 22, 8, 0, 33, 8, 3, 44],
            "起点按前面的字节数累加：1、8 都不在字边界上"
        );
        assert_eq!(packed.entry_count(), 4);
    }

    #[test]
    fn a_chunk_of_empty_units_still_has_a_four_byte_input() {
        let packed = packed_checksum_chunk(&[unit(&[], 0)]);
        assert_eq!(
            packed.input_bytes,
            vec![0, 0, 0, 0],
            "wgpu 不许长度为 0 的绑定"
        );
    }

    #[test]
    fn descriptor_bytes_are_little_endian_words() {
        let packed = packed_checksum_chunk(&[unit(&[0xEE], 0x0102_0304)]);
        assert_eq!(
            packed.descriptor_bytes(),
            vec![0, 0, 0, 0, 1, 0, 0, 0, 0x04, 0x03, 0x02, 0x01]
        );
    }

    #[test]
    fn readback_records_decode_into_computed_checksums_and_verdicts() {
        let mut readback = Vec::new();
        readback.extend_from_slice(&0xE306_9283_u32.to_le_bytes());
        readback.extend_from_slice(&1_u32.to_le_bytes());
        readback.extend_from_slice(&7_u32.to_le_bytes());
        readback.extend_from_slice(&0_u32.to_le_bytes());
        assert_eq!(
            decoded_kernel_outcomes(&readback),
            Ok(vec![
                ChecksumKernelOutcome {
                    computed_crc32c: 0xE306_9283,
                    matches_the_expected_crc32c: true
                },
                ChecksumKernelOutcome {
                    computed_crc32c: 7,
                    matches_the_expected_crc32c: false
                },
            ])
        );
    }

    #[test]
    fn an_outcome_the_kernel_never_wrote_is_an_unrecognized_verdict_word() {
        let mut readback = Vec::new();
        readback.extend_from_slice(&5_u32.to_le_bytes());
        readback.extend_from_slice(&1_u32.to_le_bytes());
        readback.extend_from_slice(&outcome_buffer_before_the_kernel_writes(1));
        assert_eq!(
            decoded_kernel_outcomes(&readback),
            Err(UnrecognizedVerdictWord {
                entry_position: 1,
                verdict_word: OUTCOME_NOT_WRITTEN_FILL_WORD
            }),
            "位置 1 的那一条还是填充字：核没写到它"
        );
    }

    #[test]
    fn the_cpu_reference_outcome_uses_the_core_crc32c() {
        assert_eq!(
            cpu_reference_outcome(b"123456789", 0xE306_9283),
            ChecksumKernelOutcome {
                computed_crc32c: 0xE306_9283,
                matches_the_expected_crc32c: true
            },
            "CRC-32C 的检验值"
        );
        assert!(!cpu_reference_outcome(b"123456789", 0).matches_the_expected_crc32c);
    }

    #[test]
    fn the_two_kernels_are_separate_sources_with_their_own_entry_points() {
        assert!(
            TABLE_DRIVEN_KERNEL_SOURCE.contains(&format!("fn {TABLE_DRIVEN_KERNEL_ENTRY_POINT}("))
        );
        assert!(BITWISE_KERNEL_SOURCE.contains(&format!("fn {BITWISE_KERNEL_ENTRY_POINT}(")));
        assert!(
            !BITWISE_KERNEL_SOURCE.contains(TABLE_DRIVEN_KERNEL_ENTRY_POINT)
                && !TABLE_DRIVEN_KERNEL_SOURCE.contains(BITWISE_KERNEL_ENTRY_POINT),
            "两份核不互相引用"
        );
    }
}
