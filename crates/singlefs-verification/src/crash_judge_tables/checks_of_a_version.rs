//! 每个版本的检查字：CPU 先按落点的类算好校验和与单元槽的标志位，GPU 只查表。

use super::*;

/// 系统配置落点的检查字里「整槽校验和字段对得上」那一字的下标（[`version_checks`] 系统配置那一臂的头一字；GPU 内核读成切片的 `c0`）。
pub(super) const SYSTEM_CONFIGURATION_SLOT_CHECKSUM_HOLDS_CHECK: usize = 0;

/// 每一类落点的检查字。
///
/// - 系统配置：`[整槽校验和字段对得上, 0, 0, 0]`。
/// - 根槽：`[整槽校验和字段对得上（罩整个槽宽）, 0, 0, 0]`。
/// - journal 记录：`[整条校验和字段对得上, 载荷 CRC 对得上（点名项区在 4096 之内）, 反向链值 = CRC-32C(头 496 字节、校验和字段按 0), 0]`。
/// - 单元槽：`[前 16 KiB 的 CRC-32C, 前 32 KiB 的 CRC-32C（落点只有 16 KiB 时 0）, 标志位, 0]`；标志位：位 0 头校验和按自述的类过、
///   位 1 载荷 CRC 按自述的类过、位 2 落点有 32 KiB、位 3 类身份段之后的 29 字节预留位全 0。类从偏移 6 读，码 2 的 key 宽从偏移 51 读。
#[must_use]
pub fn version_checks(kind: JudgeLocationKind, bytes: &[u8]) -> [u32; 4] {
    match kind {
        JudgeLocationKind::SystemConfiguration => [
            u32::from(wide_checksum_field_holds(
                bytes,
                usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096"),
                155,
            )),
            0,
            0,
            0,
        ],
        JudgeLocationKind::RootSlot => {
            let holds = bytes.len() >= usize::try_from(ROOT_RECORD_BYTES).expect("466")
                && wide_checksum_field_holds(bytes, bytes.len(), 138);
            [u32::from(holds), 0, 0, 0]
        }
        JudgeLocationKind::JournalRecord => {
            let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
            let header_bytes =
                usize::try_from(JOURNAL_HEADER_BYTES).expect("记录头 496 字节装得进 usize");
            let header_holds = wide_checksum_field_holds(bytes, record_bytes, 46);
            let named_count = usize::try_from(u32::from_le_bytes(
                bytes[12..16].try_into().expect("4 字节"),
            ))
            .expect("点名项数");
            let payload_end = header_bytes.saturating_add(named_count.saturating_mul(56));
            let payload_holds = payload_end <= record_bytes
                && crc32_castagnoli(&bytes[header_bytes..payload_end])
                    == u32::from_le_bytes(bytes[95..99].try_into().expect("4 字节"));
            let mut header = bytes[..header_bytes].to_vec();
            header[46..78].fill(0);
            [
                u32::from(header_holds),
                u32::from(payload_holds),
                crc32_castagnoli(&header),
                0,
            ]
        }
        JudgeLocationKind::UnitSlot => {
            let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
            let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
            let crc_16k = if bytes.len() >= node_bytes {
                crc32_castagnoli(&bytes[..node_bytes])
            } else {
                0
            };
            let has_32k = bytes.len() >= data_unit_bytes;
            let crc_32k = if has_32k {
                crc32_castagnoli(&bytes[..data_unit_bytes])
            } else {
                0
            };
            let reserved = usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29");
            let (header_end, unit_end, payload_crc_offset): (Option<usize>, usize, usize) =
                match bytes.get(6).copied() {
                    Some(1) => (
                        Some(usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105")),
                        data_unit_bytes,
                        101,
                    ),
                    Some(2) => {
                        let key_width = usize::from(bytes.get(51).copied().unwrap_or(0));
                        let header_end = 86 + 2 * key_width;
                        (Some(header_end), node_bytes, header_end - 10)
                    }
                    Some(3) => (Some(107), data_unit_bytes, 89),
                    _ => (None, 0, 0),
                };
            let mut flags = 0u32;
            if let Some(header_end) = header_end {
                if bytes.len() >= unit_end {
                    if wide_checksum_field_holds(bytes, header_end, 10) {
                        flags |= 1;
                    }
                    let stored = u32::from_le_bytes(
                        bytes[payload_crc_offset..payload_crc_offset + 4]
                            .try_into()
                            .expect("4 字节"),
                    );
                    if crc32_castagnoli(&bytes[header_end..unit_end]) == stored {
                        flags |= 2;
                    }
                    if bytes[header_end..header_end + reserved]
                        .iter()
                        .all(|byte| *byte == 0)
                    {
                        flags |= 8;
                    }
                    // 位 4：码 1 声明长度之后到单元末尾全 0（`unit::data_unit_payload` 的补齐判定）
                    if bytes[6] == 1 {
                        let declared = usize::from(u16::from_le_bytes([bytes[8], bytes[9]]));
                        let payload_start = header_end + reserved;
                        if payload_start + declared <= unit_end
                            && bytes[payload_start + declared..unit_end]
                                .iter()
                                .all(|byte| *byte == 0)
                        {
                            flags |= 16;
                        }
                    }
                }
            }
            if has_32k {
                flags |= 4;
            }
            [crc_16k, crc_32k, flags, unit_layout_checks(bytes)]
        }
    }
}

/// 单元槽版本的第四个检查字：池级 checker 按**引用它的那一方期望的类**的字段表判头（`walk.rs` 的 `judge_unit_header`），
/// 与单元自述的类无关，所以三种布局各算一遍：位 4L + 0 头校验和过、4L + 1 载荷 CRC 过、4L + 2 明文头之后 29 字节预留位全 0、
/// 4L + 3 声明长度之后到单元末尾全 0；L = 0 按码 1 的布局（头 105、载荷 CRC 101、载荷 134 起、单元 32768）、
/// L = 1 按码 2（头 86 + 2k、载荷 CRC 76 + 2k、条目 115 + 2k 起、单元 16384，k 取偏移 51）、L = 2 按码 3（107、89、136、32768）。
/// 落点不够那种布局的单元长时那一组位全 0。
#[must_use]
pub fn unit_layout_checks(bytes: &[u8]) -> u32 {
    let mut flags = 0u32;
    let key_width = usize::from(bytes.get(51).copied().unwrap_or(0));
    let layouts: [(usize, usize, usize, usize); 3] = [
        (
            105,
            101,
            134,
            usize::try_from(DATA_UNIT_BYTES).expect("32768"),
        ),
        (
            86 + 2 * key_width,
            76 + 2 * key_width,
            115 + 2 * key_width,
            usize::try_from(NODE_BYTES).expect("16384"),
        ),
        (
            107,
            89,
            136,
            usize::try_from(DATA_UNIT_BYTES).expect("32768"),
        ),
    ];
    let reserved = usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29");
    for (layout, (header_end, payload_crc_offset, payload_start, unit_end)) in
        layouts.into_iter().enumerate()
    {
        if bytes.len() < unit_end || header_end + reserved > unit_end {
            continue;
        }
        let bit = u32::try_from(4 * layout).expect("位号");
        if wide_checksum_field_holds(bytes, header_end, 10) {
            flags |= 1 << bit;
        }
        let stored = u32::from_le_bytes(
            bytes[payload_crc_offset..payload_crc_offset + 4]
                .try_into()
                .expect("4 字节"),
        );
        if crc32_castagnoli(&bytes[header_end..unit_end]) == stored {
            flags |= 1 << (bit + 1);
        }
        if bytes[header_end..header_end + reserved]
            .iter()
            .all(|byte| *byte == 0)
        {
            flags |= 1 << (bit + 2);
        }
        let declared = usize::from(u16::from_le_bytes([bytes[8], bytes[9]]));
        let padding_start = payload_start + declared;
        if padding_start <= unit_end && bytes[padding_start..unit_end].iter().all(|byte| *byte == 0)
        {
            flags |= 1 << (bit + 3);
        }
    }
    flags
}
