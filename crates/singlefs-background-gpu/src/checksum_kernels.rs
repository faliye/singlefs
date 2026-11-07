//! 扫描里的校验和核对（D26（后台整理与放置回收） 已定项 13 第 1 条点名的三段里的「甲」）：两份独立写的 WGSL 核（第 8 条，不共用核函数），
//! 各自对同一批单元字节算 CRC-32C 并与期望值比。这里是两份核共用的缓冲布局、打包与回读解码，都是纯函数；起 GPU 的在 `background_gpu`。
//!
//! 缓冲布局（两份核按同一个绑定号读写，各有各的输出缓冲）：
//! - 绑定 0，uniform：这一段的条数（一个 `u32`，缓冲占 [`PARAMETER_BUFFER_BYTES`]）。
//! - 绑定 1，只读存储：这一段每条的单元字节首尾相接，不对齐、不留空，末尾补 0 到 4 字节的整数倍；着色器按小端 `u32` 读。
//! - 绑定 2，只读存储：每条一项描述，三个 `u32`：在输入里的起点字节、字节数、期望的 CRC-32C。
//! - 绑定 3，读写存储：每条一项结果，两个 `u32`：算出的 CRC-32C、判定字（1 是与期望相同，0 是不同）。
//!   开算之前整块填 [`OUTCOME_NOT_WRITTEN_FILL_WORD`]：核没写到的那一条读回来判定字不是 0 也不是 1，按 GPU 出错交回。
//!
//! 一个单元单独成一段都装不下这张卡此刻剩下的地方时切成几片（`memory_limit_and_chunks::pieces_cut_to_fit_on_the_gpu`），每片当一条交两份核，
//! 核照常算这一片的 CRC-32C；再把各片的值合成整单元的值、与期望值比（[`checksum_outcomes_of_the_units_cut_into_pieces`]，用户 2026-10-28 定
//! 「装不下应该自适应的分片」）。合法也是两份、各写各的（第 8 条：两份核不共用核函数）：查表那一份核的各片按「补零字节的算子矩阵」合
//! （[`crc32c_of_two_pieces_by_the_zero_operator_matrices`]），逐位移那一份核的各片按「乘 x 的 8n 次方再取模」合
//! （[`crc32c_of_two_pieces_by_multiplying_a_power_of_the_polynomial_variable`]）；CPU 参照照旧对整单元一次算（核心层的 CRC-32C），与两种合法都不共用代码。

use crate::memory_limit_and_chunks::{ChunkFootprint, PieceOfAnEntry};
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

/// 校验和核对一段里最小的一片在卡上合占的字节数：一个字节的单元（补到一个整字）连同一段的固定占用。
#[must_use]
pub fn bytes_of_the_smallest_checksum_piece() -> u64 {
    CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(1, 1)
}

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

/// CRC-32C（Castagnoli）反射多项式。两种合法各用它，不读核心层的实现。
const CASTAGNOLI_REFLECTED_POLYNOMIAL: u32 = 0x82F6_3B78;

/// 一个 32×32 的 GF(2) 矩阵（每个字是一列）乘一个寄存器值：寄存器为 1 的那几位对应的列异或起来。
fn zero_operator_applied(zero_operator: &[u32; 32], register: u32) -> u32 {
    let mut product = 0;
    let mut remaining_bits = register;
    let mut column = 0;
    while remaining_bits != 0 {
        if remaining_bits & 1 == 1 {
            product ^= zero_operator[column];
        }
        remaining_bits >>= 1;
        column += 1;
    }
    product
}

/// 算子自乘一次：喂 k 个零位的算子变成喂 2k 个零位的算子。
fn zero_operator_squared(zero_operator: &[u32; 32]) -> [u32; 32] {
    std::array::from_fn(|column| zero_operator_applied(zero_operator, zero_operator[column]))
}

/// 前一片的 CRC-32C 与后一片的 CRC-32C 合成两片首尾相接的 CRC-32C（查表那一份核的各片用它合）：把前一片的值当寄存器，
/// 喂后一片那么多个零字节（喂零的算子按二进制自乘得到，不真喂），再异或后一片的值。CRC-32C 的初值与末了的异或在这一式里互相抵掉。
///
/// 循环：`second_length_in_bytes` 每轮右移两位，至多 32 轮；跨轮带两个算子与前一片的寄存器。
#[must_use]
pub fn crc32c_of_two_pieces_by_the_zero_operator_matrices(
    crc32c_of_the_first_piece: u32,
    crc32c_of_the_second_piece: u32,
    second_length_in_bytes: u64,
) -> u32 {
    if second_length_in_bytes == 0 {
        return crc32c_of_the_first_piece;
    }
    let mut operator_of_one_zero_bit = [0_u32; 32];
    operator_of_one_zero_bit[0] = CASTAGNOLI_REFLECTED_POLYNOMIAL;
    for (column, operator_column) in operator_of_one_zero_bit.iter_mut().enumerate().skip(1) {
        *operator_column = 1 << (column - 1);
    }
    let operator_of_two_zero_bits = zero_operator_squared(&operator_of_one_zero_bit);
    let mut operator_of_an_odd_power = zero_operator_squared(&operator_of_two_zero_bits);
    let mut register = crc32c_of_the_first_piece;
    let mut zero_bytes_left = second_length_in_bytes;
    loop {
        let operator_of_an_even_power = zero_operator_squared(&operator_of_an_odd_power);
        if zero_bytes_left & 1 == 1 {
            register = zero_operator_applied(&operator_of_an_even_power, register);
        }
        zero_bytes_left >>= 1;
        if zero_bytes_left == 0 {
            break;
        }
        operator_of_an_odd_power = zero_operator_squared(&operator_of_an_even_power);
        if zero_bytes_left & 1 == 1 {
            register = zero_operator_applied(&operator_of_an_odd_power, register);
        }
        zero_bytes_left >>= 1;
        if zero_bytes_left == 0 {
            break;
        }
    }
    register ^ crc32c_of_the_second_piece
}

/// 反射表示下两个多项式相乘再模 CRC-32C 多项式：最高位是 x 的 0 次方，最低位是 x 的 31 次方。
///
/// 循环：`multiplier` 从最高位往下看，至多 32 轮；跨轮带乘积与被乘数乘过的那几个 x。
fn product_modulo_the_polynomial(multiplier: u32, multiplicand: u32) -> u32 {
    let mut product = 0;
    let mut shifted_multiplicand = multiplicand;
    for bit_from_the_top in 0..32 {
        if multiplier & (1 << (31 - bit_from_the_top)) != 0 {
            product ^= shifted_multiplicand;
        }
        shifted_multiplicand = if shifted_multiplicand & 1 == 1 {
            (shifted_multiplicand >> 1) ^ CASTAGNOLI_REFLECTED_POLYNOMIAL
        } else {
            shifted_multiplicand >> 1
        };
    }
    product
}

/// x 的（8 × `zero_bytes`）次方模 CRC-32C 多项式，反射表示：从 x 的 8 次方起按二进制自乘。
///
/// 循环：`zero_bytes` 每轮右移一位，至多 64 轮。
fn power_of_the_polynomial_variable_for_zero_bytes(zero_bytes: u64) -> u32 {
    const POLYNOMIAL_VARIABLE_TO_THE_POWER_ZERO: u32 = 1 << 31;
    const POLYNOMIAL_VARIABLE_TO_THE_POWER_EIGHT: u32 = 1 << (31 - 8);
    let mut power = POLYNOMIAL_VARIABLE_TO_THE_POWER_ZERO;
    let mut square = POLYNOMIAL_VARIABLE_TO_THE_POWER_EIGHT;
    let mut zero_bytes_left = zero_bytes;
    while zero_bytes_left != 0 {
        if zero_bytes_left & 1 == 1 {
            power = product_modulo_the_polynomial(square, power);
        }
        square = product_modulo_the_polynomial(square, square);
        zero_bytes_left >>= 1;
    }
    power
}

/// 前一片的 CRC-32C 与后一片的 CRC-32C 合成两片首尾相接的 CRC-32C（逐位移那一份核的各片用它合）：前一片的值乘 x 的
/// （8 × 后一片字节数）次方、模多项式，再异或后一片的值。与 [`crc32c_of_two_pieces_by_the_zero_operator_matrices`] 各写各的。
#[must_use]
pub fn crc32c_of_two_pieces_by_multiplying_a_power_of_the_polynomial_variable(
    crc32c_of_the_first_piece: u32,
    crc32c_of_the_second_piece: u32,
    second_length_in_bytes: u64,
) -> u32 {
    product_modulo_the_polynomial(
        power_of_the_polynomial_variable_for_zero_bytes(second_length_in_bytes),
        crc32c_of_the_first_piece,
    ) ^ crc32c_of_the_second_piece
}

/// 一段里的几片当成几条单元交给两份核：每片的字节取它那一个单元里罩住的那一截，期望值照它那一个单元的（切开的单元各片的判定不用，合完再比）。
///
/// # Panics
/// 一片罩出它那一个单元的字节：片由同一批单元的字节数切出。
#[must_use]
pub fn units_of_the_pieces<'unit, EntryKey>(
    units: &[UnitChecksumToVerify<'unit, EntryKey>],
    pieces: &[PieceOfAnEntry],
) -> Vec<UnitChecksumToVerify<'unit, ()>> {
    pieces
        .iter()
        .map(|piece| {
            let unit = &units[piece.entry_position];
            let piece_start =
                usize::try_from(piece.input_bytes.start).expect("片的起点在单元里，装得进 usize");
            let piece_end =
                usize::try_from(piece.input_bytes.end).expect("片的终点在单元里，装得进 usize");
            UnitChecksumToVerify {
                entry_key: (),
                unit_bytes: &unit.unit_bytes[piece_start..piece_end],
                expected_crc32c: unit.expected_crc32c,
            }
        })
        .collect()
}

/// 一批单元按切好的几段（每段几片，`memory_limit_and_chunks::pieces_cut_to_fit_on_the_gpu` 切的）逐段交两份核
/// （`both_kernel_outcomes_of_a_chunk`，交回每片两份的结果，查表那一份在前），再按单元接回：整条一片的照核交回的；切成几片的，
/// 查表那一份的各片按 [`crc32c_of_two_pieces_by_the_zero_operator_matrices`]、逐位移那一份的各片按
/// [`crc32c_of_two_pieces_by_multiplying_a_power_of_the_polynomial_variable`] 依次合成整单元的值，各自与期望值比。交回每个单元两份的结果，按 `units` 的次序。
///
/// # Errors
/// 某一段交不成：照原样交回，后面的段不交。
///
/// # Panics
/// 片没有按单元的次序、单元里的字节次序一片不漏地罩住每个单元；一段交回的结果数与它的片数不等。
pub fn checksum_outcomes_of_the_units_cut_into_pieces<EntryKey, SubmissionFailure>(
    units: &[UnitChecksumToVerify<'_, EntryKey>],
    chunks_of_pieces: &[Vec<PieceOfAnEntry>],
    mut both_kernel_outcomes_of_a_chunk: impl FnMut(
        &[UnitChecksumToVerify<'_, ()>],
    ) -> Result<
        Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>,
        SubmissionFailure,
    >,
) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, SubmissionFailure> {
    let mut piece_outcomes: Vec<(
        PieceOfAnEntry,
        (ChecksumKernelOutcome, ChecksumKernelOutcome),
    )> = Vec::new();
    for pieces in chunks_of_pieces {
        let outcomes = both_kernel_outcomes_of_a_chunk(&units_of_the_pieces(units, pieces))?;
        assert_eq!(outcomes.len(), pieces.len(), "两份核每片各交一个结果");
        piece_outcomes.extend(pieces.iter().cloned().zip(outcomes));
    }
    let mut unit_outcomes = Vec::with_capacity(units.len());
    let mut next_piece = 0;
    for (entry_position, unit) in units.iter().enumerate() {
        let unit_length = u64::try_from(unit.unit_bytes.len()).expect("装得进 u64");
        let first_piece = next_piece;
        let mut covered_up_to = 0;
        while next_piece < piece_outcomes.len()
            && piece_outcomes[next_piece].0.entry_position == entry_position
        {
            assert_eq!(
                piece_outcomes[next_piece].0.input_bytes.start, covered_up_to,
                "一个单元的片按字节的次序首尾相接"
            );
            covered_up_to = piece_outcomes[next_piece].0.input_bytes.end;
            next_piece += 1;
        }
        assert!(
            next_piece > first_piece && covered_up_to == unit_length,
            "每个单元的片一片不漏地罩住它的字节"
        );
        let pieces_of_the_unit = &piece_outcomes[first_piece..next_piece];
        if let [(_whole_unit, outcomes_of_the_whole_unit)] = pieces_of_the_unit {
            unit_outcomes.push(*outcomes_of_the_whole_unit);
            continue;
        }
        let mut table_driven_crc32c = pieces_of_the_unit[0].1 .0.computed_crc32c;
        let mut bitwise_crc32c = pieces_of_the_unit[0].1 .1.computed_crc32c;
        for (piece, (table_driven, bitwise)) in &pieces_of_the_unit[1..] {
            table_driven_crc32c = crc32c_of_two_pieces_by_the_zero_operator_matrices(
                table_driven_crc32c,
                table_driven.computed_crc32c,
                piece.input_byte_count(),
            );
            bitwise_crc32c = crc32c_of_two_pieces_by_multiplying_a_power_of_the_polynomial_variable(
                bitwise_crc32c,
                bitwise.computed_crc32c,
                piece.input_byte_count(),
            );
        }
        unit_outcomes.push((
            ChecksumKernelOutcome {
                computed_crc32c: table_driven_crc32c,
                matches_the_expected_crc32c: table_driven_crc32c == unit.expected_crc32c,
            },
            ChecksumKernelOutcome {
                computed_crc32c: bitwise_crc32c,
                matches_the_expected_crc32c: bitwise_crc32c == unit.expected_crc32c,
            },
        ));
    }
    assert_eq!(
        next_piece,
        piece_outcomes.len(),
        "每一片都属于这一批的某个单元"
    );
    Ok(unit_outcomes)
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
    fn the_chunk_of_empty_units_still_has_a_four_byte_input() {
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
    fn the_table_driven_and_bitwise_checksum_kernels_are_separate_sources_with_their_own_entry_points(
    ) {
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

    /// 用例里的一串伪随机字节。
    fn bytes_for_the_test(length: usize, seed: u64) -> Vec<u8> {
        let mut state = seed ^ 0x9E37_79B9_7F4A_7C15;
        (0..length)
            .map(|_position| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                state.to_le_bytes()[7]
            })
            .collect()
    }

    /// 两种合法各自把任意切点两边的 CRC-32C 合成整串的 CRC-32C（核心层对整串一次算的值）：切点取开头、中间、末尾与几种长度。
    #[test]
    fn both_ways_of_combining_two_pieces_give_the_crc32c_of_the_whole_bytes() {
        for (length, seed) in [(0_usize, 1_u64), (1, 2), (9, 3), (4096, 4), (32768 + 3, 5)] {
            let bytes = bytes_for_the_test(length, seed);
            let whole = singlefs_core::checksum::crc32_castagnoli(&bytes);
            for split_at in [0, length / 3, length / 2, length.saturating_sub(1), length] {
                let (first, second) = bytes.split_at(split_at);
                let crc32c_of_the_first = singlefs_core::checksum::crc32_castagnoli(first);
                let crc32c_of_the_second = singlefs_core::checksum::crc32_castagnoli(second);
                let second_length = u64::try_from(second.len()).expect("装得进 u64");
                assert_eq!(
                    crc32c_of_two_pieces_by_the_zero_operator_matrices(
                        crc32c_of_the_first,
                        crc32c_of_the_second,
                        second_length
                    ),
                    whole,
                    "补零算子那一种：长 {length}、切在 {split_at}"
                );
                assert_eq!(
                    crc32c_of_two_pieces_by_multiplying_a_power_of_the_polynomial_variable(
                        crc32c_of_the_first,
                        crc32c_of_the_second,
                        second_length
                    ),
                    whole,
                    "乘 x 的幂那一种：长 {length}、切在 {split_at}"
                );
            }
        }
    }

    /// 两份核在 CPU 上的替身：每片照 CPU 参照算（核照常算一片的 CRC-32C、与期望值比）。
    fn both_kernels_on_the_cpu(
        piece_units: &[UnitChecksumToVerify<'_, ()>],
    ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, ()> {
        Ok(piece_units
            .iter()
            .map(|piece| {
                let outcome = cpu_reference_outcome(piece.unit_bytes, piece.expected_crc32c);
                (outcome, outcome)
            })
            .collect())
    }

    /// 用户 2026-10-28 定「装不下应该自适应的分片」：上限 1 KiB 时 4095、4096、32771 字节的单元切成几片交（两份核在 CPU 上的替身），
    /// 各片的值两份各按自己的合法合成，与 CPU 参照对整单元的判定逐条相同；期望值一对一错，判定两边都走到。
    #[test]
    fn units_cut_into_pieces_under_a_small_memory_limit_match_the_cpu_reference_on_every_unit() {
        let unit_bytes: Vec<Vec<u8>> = [7_usize, 4095, 4096, 0, 32771, 1]
            .iter()
            .enumerate()
            .map(|(seed, length)| bytes_for_the_test(*length, u64::try_from(seed).expect("小数")))
            .collect();
        let units: Vec<UnitChecksumToVerify<'_, usize>> = unit_bytes
            .iter()
            .enumerate()
            .map(|(entry_key, bytes)| {
                let correct = singlefs_core::checksum::crc32_castagnoli(bytes);
                UnitChecksumToVerify {
                    entry_key,
                    unit_bytes: bytes,
                    expected_crc32c: if entry_key % 2 == 0 {
                        correct
                    } else {
                        correct ^ 1
                    },
                }
            })
            .collect();
        let lengths: Vec<u64> = unit_bytes
            .iter()
            .map(|bytes| u64::try_from(bytes.len()).expect("装得进 u64"))
            .collect();
        let chunks_of_pieces = crate::memory_limit_and_chunks::pieces_cut_to_fit_on_the_gpu(
            &lengths,
            CHECKSUM_CHUNK_FOOTPRINT,
            crate::memory_limit_and_chunks::ChunkBoundsOfAGpu {
                memory_limit: Some(singlefs_core::background_compute::GpuMemoryInBytes(1024)),
                largest_storage_binding_in_bytes: 1 << 30,
                largest_entry_count_per_dispatch: 1 << 20,
            },
        )
        .expect("1 KiB 装得下最小的一片");
        assert!(
            chunks_of_pieces.iter().flatten().count() > units.len() + 30,
            "有单元切成了几片"
        );
        let outcomes = checksum_outcomes_of_the_units_cut_into_pieces(
            &units,
            &chunks_of_pieces,
            both_kernels_on_the_cpu,
        )
        .expect("替身不出错");
        assert_eq!(
            outcomes,
            units
                .iter()
                .map(|unit| {
                    let reference = cpu_reference_outcome(unit.unit_bytes, unit.expected_crc32c);
                    (reference, reference)
                })
                .collect::<Vec<_>>(),
            "切成几片再合，两份照样逐条与 CPU 参照对整单元的判定相同"
        );
        assert_eq!(
            outcomes
                .iter()
                .filter(|(table_driven, _bitwise)| table_driven.matches_the_expected_crc32c)
                .count(),
            3,
            "一半对、一半错"
        );
    }
}
