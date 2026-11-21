//! 一批单元的两道校验和：头校验和与载荷 CRC 在卡上算（CRC-32C 的内核与它一次要的显存）与 CPU 参照，两边同一套切段、同一个判法。

use super::*;

/// 一个 GPU 槽装一段字节：最大 32 KiB（数据单元的长度），短的按长度截断。
pub(super) const SLOT_BYTES: usize = 32_768;
pub(super) const UNIT_CHECKSUM_KERNEL_WORKGROUP_SIZE: usize = 64;
pub(super) const RESULT_ENTRY_BYTES: usize = 8;
pub(super) const CRC32_CASTAGNOLI_SHADER_SOURCE: &str = r#"
struct Params { count: u32, reserved0: u32, reserved1: u32, reserved2: u32, };
struct ResultEntry { crc: u32, local_offset: u32, };
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> slot_words: array<u32>;
@group(0) @binding(2) var<storage, read> slot_lengths: array<u32>;
@group(0) @binding(3) var<storage, read_write> results: array<ResultEntry>;
const CRC32C_REFLECTED_POLYNOMIAL: u32 = 0x82F63B78u;
const CRC32C_INITIAL_VALUE: u32 = 0xFFFFFFFFu;
const WORDS_PER_SLOT: u32 = 8192u;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let local_offset = global_id.x;
    if (local_offset >= params.count) { return; }
    let length_in_bytes = slot_lengths[local_offset];
    let base_word = local_offset * WORDS_PER_SLOT;
    var crc_register: u32 = CRC32C_INITIAL_VALUE;
    var byte_index: u32 = 0u;
    loop {
        if (byte_index >= length_in_bytes) { break; }
        let word = slot_words[base_word + byte_index / 4u];
        let current_byte = (word >> (8u * (byte_index % 4u))) & 0xFFu;
        crc_register = crc_register ^ current_byte;
        var bit_index: u32 = 0u;
        loop {
            if (bit_index >= 8u) { break; }
            if ((crc_register & 1u) == 1u) {
                crc_register = (crc_register >> 1u) ^ CRC32C_REFLECTED_POLYNOMIAL;
            } else {
                crc_register = crc_register >> 1u;
            }
            bit_index = bit_index + 1u;
        }
        byte_index = byte_index + 1u;
    }
    results[local_offset].crc = crc_register ^ CRC32C_INITIAL_VALUE;
    results[local_offset].local_offset = local_offset;
}
"#;

/// 在卡上算 `range_count` 段字节的 CRC-32C（[`GpuCard::crc32c_of_ranges`]）一次要多少显存（MiB，向上取整）：每段一个 32 KiB 的槽、
/// 一个长度字、一条结果与它的回读，加一份派活参数。核一批单元的两道校验和（[`check_unit_checksums_on_gpu`]）每个单元两段。
/// 用例向卡池借卡时按它借（`gpu_test_start::GpuCardPool::lease_card_or_report_not_run`）。
#[must_use]
pub fn mebibytes_crc32c_of_ranges_needs(range_count: usize) -> u64 {
    bytes_crc32c_of_ranges_needs(range_count).div_ceil(1024 * 1024)
}

/// 同 [`mebibytes_crc32c_of_ranges_needs`]，按字节、不取整：[`GpuCard::crc32c_of_ranges`] 一次建的五个缓冲
/// （派活参数 16 字节、每段一个 32 KiB 的槽、一个长度字、一条结果与它的回读各 8 字节）。崩溃放量第 ① 段由 CPU 判、
/// 单元校验和交卡对拍时按它算几条线程同时派活占多少（`crate::crash_amplification` 文件头第 5 条）。
#[must_use]
pub fn bytes_crc32c_of_ranges_needs(range_count: usize) -> u64 {
    const PARAMETER_BYTES: usize = 16;
    let bytes = range_count * (SLOT_BYTES + 4 + 2 * RESULT_ENTRY_BYTES) + PARAMETER_BYTES;
    u64::try_from(bytes).expect("装得进 u64")
}

/// 一个单元两道校验和的核对结果。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitChecksumVerdict {
    BothHold,
    /// 类标签不认得、长度不对：不是这一截能判的，交回 CPU 那一整套去判。
    NotAUnitThisCheckReads,
    HeaderChecksumMismatch,
    PayloadCrcMismatch,
}

/// 一个单元按它头里的布局切成要算 CRC 的两段：头那一段（头校验和字段清零之后的副本）与载荷那一段，连同头里存着的两道校验和。
/// 类标签不认得、长度不对的交回 None（[`UnitChecksumVerdict::NotAUnitThisCheckReads`]）。
struct UnitChecksumRanges<'unit> {
    header_with_its_checksum_field_zeroed: Vec<u8>,
    payload: &'unit [u8],
    stored_header_checksum: u32,
    stored_payload_crc: u32,
}

fn unit_checksum_ranges(unit: &[u8]) -> Option<UnitChecksumRanges<'_>> {
    let layout = unit_checksum_layout(unit).filter(|layout| {
        u64::try_from(unit.len()).ok() == Some(layout.expected_length_in_bytes)
            && layout.header_end <= unit.len()
    })?;
    let mut header_with_its_checksum_field_zeroed = unit[..layout.header_end].to_vec();
    let field = UNIT_HEADER_CHECKSUM_BYTE_OFFSET..UNIT_HEADER_CHECKSUM_BYTE_OFFSET + 32;
    let stored_header_checksum = u32::from_le_bytes(
        unit[field.start..field.start + 4]
            .try_into()
            .expect("4 字节"),
    );
    header_with_its_checksum_field_zeroed[field].fill(0);
    let stored_payload_crc = u32::from_le_bytes(
        unit[layout.payload_crc_offset..layout.payload_crc_offset + 4]
            .try_into()
            .expect("4 字节"),
    );
    Some(UnitChecksumRanges {
        header_with_its_checksum_field_zeroed,
        payload: &unit[layout.header_end..],
        stored_header_checksum,
        stored_payload_crc,
    })
}

/// 两道算出来的 CRC 与头里存着的比，交回这个单元的核对结果。
fn unit_checksum_verdict(
    ranges: &UnitChecksumRanges<'_>,
    computed_header_checksum: u32,
    computed_payload_crc: u32,
) -> UnitChecksumVerdict {
    if computed_header_checksum != ranges.stored_header_checksum {
        UnitChecksumVerdict::HeaderChecksumMismatch
    } else if computed_payload_crc != ranges.stored_payload_crc {
        UnitChecksumVerdict::PayloadCrcMismatch
    } else {
        UnitChecksumVerdict::BothHold
    }
}

/// 一批单元在一张卡上核两道校验和（CRC 在卡上算）。CPU 参照另是一趟（[`check_unit_checksums_on_the_cpu`]）：
/// 两边各判各的、由对拍用例逐个比（一段里 CPU 不既跑数据又对拍），两边的输入都从库里读回。
#[must_use]
pub fn check_unit_checksums_on_gpu(card: &GpuCard, units: &[&[u8]]) -> Vec<UnitChecksumVerdict> {
    let plan: Vec<Option<UnitChecksumRanges<'_>>> = units
        .iter()
        .map(|unit| unit_checksum_ranges(unit))
        .collect();
    let ranges: Vec<&[u8]> = plan
        .iter()
        .flatten()
        .flat_map(|unit_ranges| {
            [
                unit_ranges.header_with_its_checksum_field_zeroed.as_slice(),
                unit_ranges.payload,
            ]
        })
        .collect();
    let gpu = card.crc32c_of_ranges(&ranges);
    // 每个切得开的单元在卡上算两段 CRC（头那一段、载荷那一段），按切得开的单元的次序挨着回来
    let mut next_crc = 0usize;
    plan.iter()
        .map(|entry| match entry {
            None => UnitChecksumVerdict::NotAUnitThisCheckReads,
            Some(unit_ranges) => {
                let (header_crc, payload_crc) = (gpu[next_crc], gpu[next_crc + 1]);
                next_crc += 2;
                unit_checksum_verdict(unit_ranges, header_crc, payload_crc)
            }
        })
        .collect()
}

/// [`check_unit_checksums_on_gpu`] 的 CPU 参照：同一套切段、同一个判法，CRC 在 CPU 上算。
#[must_use]
pub fn check_unit_checksums_on_the_cpu(units: &[&[u8]]) -> Vec<UnitChecksumVerdict> {
    units
        .iter()
        .map(|unit| match unit_checksum_ranges(unit) {
            None => UnitChecksumVerdict::NotAUnitThisCheckReads,
            Some(unit_ranges) => unit_checksum_verdict(
                &unit_ranges,
                crc32_castagnoli_table(&unit_ranges.header_with_its_checksum_field_zeroed),
                crc32_castagnoli_table(unit_ranges.payload),
            ),
        })
        .collect()
}
