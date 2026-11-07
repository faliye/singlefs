//! CRC-32C（Castagnoli）与 32 字节校验和字段（D18（块里携带什么信息） 已定项 17：
//! 多项式 0x1EDC6F41、反射实现、初值 0xFFFFFFFF、输出取反；结果小端写进字段前 4 字节，其余 28 字节恒 0）。
//! 实现走两条路之一：x86_64 上运行时查到 SSE4.2 就用 `crc32` 指令三路交错（[`CastagnoliRoute::HardwareCrcInstructionThreeWay`]），
//! 查不到或不是 x86_64 就用 slicing-by-8（[`CastagnoliRoute::SoftwareSlicingByEight`]）；两条路对同一段字节逐字节同值。
//! checker 另写一份按位的，两份对同一段字节必须给同一个数（D13（验证路线） 已定项 5），
//! checker 那一份不跟着换。
//!
//! 选路照 `.claude/rules/fs-design.md`「五条硬要求」：判据只有「这台处理器有没有 SSE4.2」一条（[`castagnoli_route_of_this_processor`]，
//! 进程活着时不变）；运行时看得出走了哪条（同一个函数交回的就是这一次走的路）；[`crc32_castagnoli_on_route`] 是只供测试的开关，
//! 无视判据走点名的那条。

use singlefs_format::WIDE_CHECKSUM_BYTES;
use vstd::prelude::*;

// 规格与引理只在 Verus 编的时候有（平时的构建把它们擦掉），按名字引会编不过，所以整个模块引进来。
use crate::bytes::*;

/// 反射后的 Castagnoli 多项式。
const CASTAGNOLI_POLYNOMIAL_REFLECTED: u32 = 0x82F6_3B78;

/// CRC 寄存器的初值，也是出来时异或的那个数（D18 已定项 17：初值 0xFFFFFFFF、输出取反）。
const CASTAGNOLI_REGISTER_INITIAL_VALUE: u32 = 0xFFFF_FFFF;

fn castagnoli_tables() -> &'static [[u32; 256]; 8] {
    static TABLES: std::sync::OnceLock<[[u32; 256]; 8]> = std::sync::OnceLock::new();
    TABLES.get_or_init(|| {
        let mut tables = [[0u32; 256]; 8];
        for byte_value in 0..256u32 {
            let mut remainder = byte_value;
            for _round in 0..8 {
                remainder = if remainder & 1 == 1 {
                    (remainder >> 1) ^ CASTAGNOLI_POLYNOMIAL_REFLECTED
                } else {
                    remainder >> 1
                };
            }
            tables[0][usize::try_from(byte_value).expect("0..256 装得进 usize")] = remainder;
        }
        for byte_value in 0..256usize {
            let mut remainder = tables[0][byte_value];
            for table_index in 1..8 {
                remainder = tables[0][usize::try_from(remainder & 0xff).expect("低 8 位")]
                    ^ (remainder >> 8);
                tables[table_index][byte_value] = remainder;
            }
        }
        tables
    })
}

fn table_index(value: u32) -> usize {
    usize::try_from(value & 0xff).expect("低 8 位装得进 usize")
}

/// 算 CRC-32C 走哪条路。封闭集合；硬件那条只在 x86_64 上有。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastagnoliRoute {
    /// `crc32` 指令（SSE4.2），每 [`HARDWARE_CRC_LANE_BYTES`] × 3 字节一块三路交错、按零字节移位合并，余下的单路。
    /// 带着「这台处理器查到了 SSE4.2」的凭据：没查到造不出这个成员。
    #[cfg(target_arch = "x86_64")]
    HardwareCrcInstructionThreeWay(CrcInstructionDetected),
    /// 查表 slicing-by-8：每 8 字节查 8 张 256 项的表，余下的逐字节查一张。
    SoftwareSlicingByEight,
}

/// 「这台处理器有 SSE4.2」的凭据：只有 [`CrcInstructionDetected::on_this_processor`] 查到了才造得出来，硬件那条路的 `unsafe` 靠它。
#[cfg(target_arch = "x86_64")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrcInstructionDetected {
    _constructed_only_after_the_runtime_detection: (),
}

#[cfg(target_arch = "x86_64")]
impl CrcInstructionDetected {
    /// 运行时查这台处理器有没有 SSE4.2（std 查一次、记下，之后每次是一次读）；有才交回凭据。
    #[must_use]
    pub fn on_this_processor() -> Option<Self> {
        std::arch::is_x86_feature_detected!("sse4.2").then_some(Self {
            _constructed_only_after_the_runtime_detection: (),
        })
    }
}

/// 这台处理器上 [`crc32_castagnoli`] 走哪条路：查到 SSE4.2 走硬件指令，否则走 slicing-by-8。
/// 判据是处理器的特性位，进程活着时不变（`.claude/rules/fs-design.md`「分支的三条约束」第 1 条）。
#[cfg(target_arch = "x86_64")]
#[must_use]
pub fn castagnoli_route_of_this_processor() -> CastagnoliRoute {
    match CrcInstructionDetected::on_this_processor() {
        Some(detected) => CastagnoliRoute::HardwareCrcInstructionThreeWay(detected),
        None => CastagnoliRoute::SoftwareSlicingByEight,
    }
}

/// 这台处理器上 [`crc32_castagnoli`] 走哪条路：不是 x86_64，只有 slicing-by-8。
#[cfg(not(target_arch = "x86_64"))]
#[must_use]
pub fn castagnoli_route_of_this_processor() -> CastagnoliRoute {
    CastagnoliRoute::SoftwareSlicingByEight
}

verus! {

/// CRC-32C 在 Verus 里是未解释函数（D13（验证路线） 已定项 14「哈希是 uninterpreted function」）：证明只用到「同一段字节同一个数」，
/// 不展开多项式。它与真实现对不对得上不归 Verus 判，归差分测试 [`tests::crc32c_matches_the_published_check_value`]（公开校验值）
/// 与 checker 那一份按位实现的交叉比（D13（验证路线） 已定项 5）；这一条与 [`crc32_castagnoli`] 的 `external_body` 各登记一行信任清单。
pub uninterp spec fn spec_crc32_castagnoli(bytes: Seq<u8>) -> u32;

/// 完整 32 位 CRC-32C，走 [`castagnoli_route_of_this_processor`] 选的那条路。
///
/// 函数体不经 Verus 验（`external_body`）：查表、硬件指令与选路都在 `verus!` 外面，被验区域只信「交回的就是那段字节的 CRC-32C」。
#[verifier::external_body]
#[must_use]
pub fn crc32_castagnoli(bytes: &[u8]) -> (crc: u32)
    ensures
        crc == spec_crc32_castagnoli(bytes@),
{
    crc32_castagnoli_on_route(castagnoli_route_of_this_processor(), bytes)
}

} // verus!

/// 完整 32 位 CRC-32C，走点名的那条路：只供测试的开关（`.claude/rules/fs-design.md`「五条硬要求」第 2 条），
/// 用例拿它在有 SSE4.2 的机器上强制走 slicing-by-8；产品代码只调 [`crc32_castagnoli`]。两条路对同一段字节交回同一个数。
#[must_use]
#[allow(
    unsafe_code,
    reason = "调带 #[target_feature(enable = \"sse4.2\")] 的函数要 unsafe；std 没有「查过特性再调」的安全封装（用户 2026-10-26 定 CRC 换硬件指令）"
)]
pub fn crc32_castagnoli_on_route(route: CastagnoliRoute, bytes: &[u8]) -> u32 {
    let register = match route {
        #[cfg(target_arch = "x86_64")]
        CastagnoliRoute::HardwareCrcInstructionThreeWay(_detected) => {
            // SAFETY: `register_after_the_bytes` 只要求这台处理器有 SSE4.2；`CrcInstructionDetected` 的字段私有，
            // 只有 `CrcInstructionDetected::on_this_processor` 在 `is_x86_feature_detected!("sse4.2")` 为真时造得出来，带着它就是查到过。
            unsafe {
                hardware_crc_instruction::register_after_the_bytes(
                    CASTAGNOLI_REGISTER_INITIAL_VALUE,
                    bytes,
                )
            }
        }
        CastagnoliRoute::SoftwareSlicingByEight => {
            software_register_after_the_bytes(CASTAGNOLI_REGISTER_INITIAL_VALUE, bytes)
        }
    };
    register ^ CASTAGNOLI_REGISTER_INITIAL_VALUE
}

/// slicing-by-8：从 `register` 起吃完 `bytes`，交回寄存器（不取反）。
fn software_register_after_the_bytes(register: u32, bytes: &[u8]) -> u32 {
    let tables = castagnoli_tables();
    let mut remainder = register;
    let (chunks, remainder_bytes) = bytes.as_chunks::<8>();
    for chunk in chunks {
        let low = u32::from_le_bytes(chunk[..4].try_into().expect("切了 4 字节")) ^ remainder;
        let high = u32::from_le_bytes(chunk[4..].try_into().expect("切了 4 字节"));
        remainder = tables[7][table_index(low)]
            ^ tables[6][table_index(low >> 8)]
            ^ tables[5][table_index(low >> 16)]
            ^ tables[4][table_index(low >> 24)]
            ^ tables[3][table_index(high)]
            ^ tables[2][table_index(high >> 8)]
            ^ tables[1][table_index(high >> 16)]
            ^ tables[0][table_index(high >> 24)];
    }
    for &byte in remainder_bytes {
        remainder = tables[0][table_index(remainder ^ u32::from(byte))] ^ (remainder >> 8);
    }
    remainder
}

/// 硬件指令那条路三路交错时每一路吃多少字节（8 的倍数）。16 KiB 节点十块三路、余 1 KiB 单路，32 KiB 数据单元二十一块、余 512 字节：
/// 块越长合并的两次移位摊得越薄，块越短余下走单路的越少；取值是推的，没量过。
#[cfg(target_arch = "x86_64")]
pub const HARDWARE_CRC_LANE_BYTES: usize = 512;

/// `crc32` 指令那条路。三路交错的合并：寄存器吃一段字节是仿射的，从 `r` 起吃完 `A` 再吃 `B` 等于
/// 「从 `r` 吃完 `A` 的寄存器再接 |B| 个零字节」异或「从 0 吃完 `B`」；三段等长，第一段从调用方的寄存器起、后两段从 0 起，
/// 合并是两次「接一路长的零字节」各异或一段。
#[cfg(target_arch = "x86_64")]
mod hardware_crc_instruction {
    use std::arch::x86_64::{_mm_crc32_u64, _mm_crc32_u8};

    use super::{software_register_after_the_bytes, table_index, HARDWARE_CRC_LANE_BYTES};

    /// 一条 `crc32q` 吃几字节。
    const BYTES_PER_INSTRUCTION_WORD: usize = 8;

    /// 一块三路交错吃几字节。
    const THREE_WAY_BLOCK_BYTES: usize = 3 * HARDWARE_CRC_LANE_BYTES;

    /// 寄存器的位数。
    const REGISTER_BITS: usize = 32;

    /// 「寄存器接 [`HARDWARE_CRC_LANE_BYTES`] 个零字节」这个 二元域上的线性变换，按寄存器的四个字节各一张 256 项的表：
    /// 第 k 张的第 v 项是只有第 k 个字节为 v 的寄存器接完零字节之后的样子，四张查出来异或就是整个寄存器接完的样子。
    /// 每一列（只有一位为 1 的寄存器）用 slicing-by-8 那条路吃零字节现算，与指令那条路不共用代码。
    fn tables_shifting_over_one_lane_of_zero_bytes() -> &'static [[u32; 256]; 4] {
        static TABLES: std::sync::OnceLock<[[u32; 256]; 4]> = std::sync::OnceLock::new();
        TABLES.get_or_init(|| {
            let zero_lane = [0u8; HARDWARE_CRC_LANE_BYTES];
            let mut columns = [0u32; REGISTER_BITS];
            for (bit, column) in columns.iter_mut().enumerate() {
                *column = software_register_after_the_bytes(1u32 << bit, &zero_lane);
            }
            let mut tables = [[0u32; 256]; 4];
            for (byte_position, table) in tables.iter_mut().enumerate() {
                for (byte_value, entry) in table.iter_mut().enumerate() {
                    // 迭代上界是一个字节的 8 位；线性：只有这几位为 1 的寄存器接完零字节，是这几位各自那一列的异或。
                    *entry = (0..8)
                        .filter(|bit_in_byte| byte_value & (1 << bit_in_byte) != 0)
                        .fold(0u32, |shifted, bit_in_byte| {
                            shifted ^ columns[8 * byte_position + bit_in_byte]
                        });
                }
            }
            tables
        })
    }

    fn shifted_over_one_lane_of_zero_bytes(register: u32) -> u32 {
        let tables = tables_shifting_over_one_lane_of_zero_bytes();
        tables[0][table_index(register)]
            ^ tables[1][table_index(register >> 8)]
            ^ tables[2][table_index(register >> 16)]
            ^ tables[3][table_index(register >> 24)]
    }

    /// 一条 `crc32q` 吃的那个 64 位字：8 字节按小端拼（指令按反射 CRC 的次序先吃低字节）。
    fn instruction_word_of(word: &[u8; BYTES_PER_INSTRUCTION_WORD]) -> u64 {
        u64::from_le_bytes(*word)
    }

    fn low_half_of_the_register(wide_register: u64) -> u32 {
        u32::try_from(wide_register).expect("crc32q 只写寄存器的低 32 位，高 32 位恒 0")
    }

    /// 从 `register` 起吃完 `bytes`，交回寄存器（不取反）：整块的三路交错，余下的单路。
    #[target_feature(enable = "sse4.2")]
    pub(super) fn register_after_the_bytes(register: u32, bytes: &[u8]) -> u32 {
        let (blocks, rest) = bytes.as_chunks::<THREE_WAY_BLOCK_BYTES>();
        let mut register_so_far = register;
        for block in blocks {
            register_so_far = register_after_one_three_way_block(register_so_far, block);
        }
        register_after_one_lane(register_so_far, rest)
    }

    /// 单路：每 8 字节一条 `crc32q`，尾巴逐字节一条 `crc32b`。
    #[target_feature(enable = "sse4.2")]
    fn register_after_one_lane(register: u32, bytes: &[u8]) -> u32 {
        let (words, tail) = bytes.as_chunks::<BYTES_PER_INSTRUCTION_WORD>();
        let mut wide_register = u64::from(register);
        for word in words {
            wide_register = _mm_crc32_u64(wide_register, instruction_word_of(word));
        }
        let mut narrow_register = low_half_of_the_register(wide_register);
        for &byte in tail {
            narrow_register = _mm_crc32_u8(narrow_register, byte);
        }
        narrow_register
    }

    /// 一块三路：块切成等长三段，三段在同一圈里各走一条 `crc32q`（指令的延迟由另两路填上），再按零字节移位合并。
    #[target_feature(enable = "sse4.2")]
    fn register_after_one_three_way_block(
        register: u32,
        block: &[u8; THREE_WAY_BLOCK_BYTES],
    ) -> u32 {
        let (first_lane, later_lanes) = block.split_at(HARDWARE_CRC_LANE_BYTES);
        let (second_lane, third_lane) = later_lanes.split_at(HARDWARE_CRC_LANE_BYTES);
        let (first_words, _) = first_lane.as_chunks::<BYTES_PER_INSTRUCTION_WORD>();
        let (second_words, _) = second_lane.as_chunks::<BYTES_PER_INSTRUCTION_WORD>();
        let (third_words, _) = third_lane.as_chunks::<BYTES_PER_INSTRUCTION_WORD>();
        let mut first_register = u64::from(register);
        let mut second_register = 0u64;
        let mut third_register = 0u64;
        // 迭代上界是一路的字数（HARDWARE_CRC_LANE_BYTES ÷ 8）；三路等长，zip 不丢字。
        for ((first_word, second_word), third_word) in
            first_words.iter().zip(second_words).zip(third_words)
        {
            first_register = _mm_crc32_u64(first_register, instruction_word_of(first_word));
            second_register = _mm_crc32_u64(second_register, instruction_word_of(second_word));
            third_register = _mm_crc32_u64(third_register, instruction_word_of(third_word));
        }
        let after_two_lanes =
            shifted_over_one_lane_of_zero_bytes(low_half_of_the_register(first_register))
                ^ low_half_of_the_register(second_register);
        shifted_over_one_lane_of_zero_bytes(after_two_lanes)
            ^ low_half_of_the_register(third_register)
    }
}

verus! {

// ===== 这一份文件用到的 singlefs-format 常量的规格（写法同 unit.rs 头上那一段）=====
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::WIDE_CHECKSUM_BYTES ] -> (wide_checksum_bytes: u64) returns 32u64;
#[allow(unsafe_code, reason = "Verus 把 assume_specification 展开成 unsafe fn 声明；只在 Verus 编的时候有，产品构建里擦掉")]
pub assume_specification[ singlefs_format::CHECKSUM_CRC32_CASTAGNOLI_BYTES ] -> (crc_bytes: u64) returns 4u64;

/// 求校验和时罩住的那一截：`[0, cover_end)`，其中校验和字段自己那 32 字节按 0 参与（I-2.4）。
pub open spec fn spec_covered_with_the_field_zeroed(bytes: Seq<u8>, cover_end: int, field_offset: int) -> Seq<u8> {
    bytes.subrange(0, field_offset) + zero_bytes(WIDE_CHECKSUM_BYTES as nat) + bytes.subrange(
        field_offset + WIDE_CHECKSUM_BYTES,
        cover_end,
    )
}

/// 32 字节校验和字段该写的样子：罩住那一截的 CRC-32C 小端 4 字节，其余 28 字节 0（D18（块里携带什么信息） 已定项 17）。
pub open spec fn spec_wide_checksum_field(bytes: Seq<u8>, cover_end: int, field_offset: int) -> Seq<u8> {
    little_endian_bytes(
        spec_crc32_castagnoli(spec_covered_with_the_field_zeroed(bytes, cover_end, field_offset)) as nat,
        singlefs_format::CHECKSUM_CRC32_CASTAGNOLI_BYTES as nat,
    ) + zero_bytes((WIDE_CHECKSUM_BYTES - singlefs_format::CHECKSUM_CRC32_CASTAGNOLI_BYTES) as nat)
}

/// 校验和字段的前提：字段整个落在罩住的那一截里，罩住的那一截不过 `bytes` 末尾。
pub open spec fn spec_wide_checksum_field_fits(bytes: Seq<u8>, cover_end: int, field_offset: int) -> bool {
    &&& 0 <= field_offset
    &&& field_offset + WIDE_CHECKSUM_BYTES <= cover_end
    &&& cover_end <= bytes.len()
}

/// 32 字节校验和字段的值：CRC-32C 小端 4 字节，其余 28 字节 0。
fn wide_checksum_field_of_the_crc(crc: u32) -> (field: [u8; 32])
    ensures
        field@ == little_endian_bytes(crc as nat, singlefs_format::CHECKSUM_CRC32_CASTAGNOLI_BYTES as nat) + zero_bytes(
            (WIDE_CHECKSUM_BYTES - singlefs_format::CHECKSUM_CRC32_CASTAGNOLI_BYTES) as nat,
        ),
{
    let mut field = [0u8; 32];
    field[0] = u8::try_from(crc % 256).expect("对 256 取余装得进一个字节");
    field[1] = u8::try_from(crc / 0x100 % 256).expect("对 256 取余装得进一个字节");
    field[2] = u8::try_from(crc / 0x1_0000 % 256).expect("对 256 取余装得进一个字节");
    field[3] = u8::try_from(crc / 0x100_0000 % 256).expect("对 256 取余装得进一个字节");
    proof {
        reveal_with_fuel(little_endian_bytes, 5);
        assert(field@ =~= little_endian_bytes(crc as nat, 4) + zero_bytes(28));
    }
    field
}

/// 32 字节校验和字段的值：把 `[field_offset, field_offset + 32)` 按 0 参与，对 `[0, cover_end)` 求 CRC-32C，
/// 小端写进前 4 字节、其余 28 字节 0（I-2.4：字段自身按 0 参与）。
///
/// # Panics
/// 字段不整个落在罩住的那一截里，或罩住的那一截过了 `bytes` 末尾（`requires` 排除的那一种）。
#[must_use]
pub fn wide_checksum_with_field_zeroed(
    bytes: &[u8],
    cover_end: usize,
    field_offset: usize,
) -> (field: [u8; 32])
    requires
        spec_wide_checksum_field_fits(bytes@, cover_end as int, field_offset as int),
    ensures
        field@ == spec_wide_checksum_field(bytes@, cover_end as int, field_offset as int),
{
    let field_width = usize::try_from(WIDE_CHECKSUM_BYTES).expect("32");
    // 罩住的那一截：字段之前照抄、字段 32 字节按 0、字段之后照抄到 cover_end（写者 new 出来就是 0，跳过的那一段保持 0）。
    let mut covered_writer = ByteWriter::new(cover_end);
    covered_writer.put(&bytes[..field_offset]);
    covered_writer.skip(field_width);
    covered_writer.put(&bytes[field_offset + field_width..cover_end]);
    let covered = covered_writer.into_bytes();
    proof {
        assert(covered@ =~= spec_covered_with_the_field_zeroed(bytes@, cover_end as int, field_offset as int));
    }
    wide_checksum_field_of_the_crc(crc32_castagnoli(&covered))
}

/// 读者那一侧：字段里那 28 字节非 0 一律判损坏（D18（块里携带什么信息） 已定项 17）。
///
/// # Panics
/// 同 [`wide_checksum_with_field_zeroed`]。
#[must_use]
pub fn wide_checksum_field_holds(bytes: &[u8], cover_end: usize, field_offset: usize) -> (holds: bool)
    requires
        spec_wide_checksum_field_fits(bytes@, cover_end as int, field_offset as int),
    ensures
        holds == (bytes@.subrange(field_offset as int, field_offset + WIDE_CHECKSUM_BYTES)
            == spec_wide_checksum_field(bytes@, cover_end as int, field_offset as int)),
{
    let field_width = usize::try_from(WIDE_CHECKSUM_BYTES).expect("32");
    let stored = &bytes[field_offset..field_offset + field_width];
    let expected = wide_checksum_with_field_zeroed(bytes, cover_end, field_offset);
    let holds = expected == stored;
    proof {
        if holds {
            assert(stored@ =~= expected@);
        } else {
            assert(stored@ != expected@);
        }
    }
    holds
}

} // verus!

#[cfg(test)]
mod tests {
    use super::*;

    /// 公开的校验值：CRC-32C("123456789") = 0xE3069283。
    #[test]
    fn crc32c_matches_the_published_check_value() {
        assert_eq!(crc32_castagnoli(b"123456789"), 0xE306_9283);
        assert_eq!(crc32_castagnoli(b""), 0);
    }

    #[test]
    fn wide_checksum_field_is_crc_then_zeroes_and_ignores_its_own_bytes() {
        let mut bytes = vec![7u8; 128];
        let field = wide_checksum_with_field_zeroed(&bytes, 128, 40);
        assert!(field[4..].iter().all(|byte| *byte == 0));
        bytes[40..72].copy_from_slice(&field);
        assert!(wide_checksum_field_holds(&bytes, 128, 40));
        bytes[100] ^= 1;
        assert!(
            !wide_checksum_field_holds(&bytes, 128, 40),
            "覆盖区内改一字节必须失配"
        );
        bytes[100] ^= 1;
        bytes[45] = 1;
        assert!(
            !wide_checksum_field_holds(&bytes, 128, 40),
            "那 28 字节非 0 判损坏"
        );
    }
}
