//! CRC-32C（Castagnoli）与 32 字节校验和字段（D18（块里携带什么信息） 已定项 17：
//! 多项式 0x1EDC6F41、反射实现、初值 0xFFFFFFFF、输出取反；结果小端写进字段前 4 字节，其余 28 字节恒 0）。
//! 实现用 slicing-by-8；checker 另写一份按位的，两份对同一段字节必须给同一个数（D13（验证路线） 已定项 5）。

use singlefs_format::{CHECKSUM_CRC32_CASTAGNOLI_BYTES, WIDE_CHECKSUM_BYTES};

/// 反射后的 Castagnoli 多项式。
const CASTAGNOLI_POLYNOMIAL_REFLECTED: u32 = 0x82F6_3B78;

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

/// 完整 32 位 CRC-32C。
#[must_use]
pub fn crc32_castagnoli(bytes: &[u8]) -> u32 {
    let tables = castagnoli_tables();
    let mut remainder = !0u32;
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
    !remainder
}

/// 32 字节校验和字段的值：把 `[field_offset, field_offset + 32)` 按 0 参与，对 `[0, cover_end)` 求 CRC-32C，
/// 小端写进前 4 字节、其余 28 字节 0（I-2.4：字段自身按 0 参与）。
#[must_use]
pub fn wide_checksum_with_field_zeroed(
    bytes: &[u8],
    cover_end: usize,
    field_offset: usize,
) -> [u8; 32] {
    let mut covered = bytes[..cover_end].to_vec();
    let field_width = usize::try_from(WIDE_CHECKSUM_BYTES).expect("32");
    covered[field_offset..field_offset + field_width].fill(0);
    let mut field = [0u8; 32];
    let crc_width = usize::try_from(CHECKSUM_CRC32_CASTAGNOLI_BYTES).expect("4");
    field[..crc_width].copy_from_slice(&crc32_castagnoli(&covered).to_le_bytes());
    field
}

/// 读者那一侧：字段里那 28 字节非 0 一律判损坏（D18（块里携带什么信息） 已定项 17）。
#[must_use]
pub fn wide_checksum_field_holds(bytes: &[u8], cover_end: usize, field_offset: usize) -> bool {
    let field_width = usize::try_from(WIDE_CHECKSUM_BYTES).expect("32");
    let stored = &bytes[field_offset..field_offset + field_width];
    wide_checksum_with_field_zeroed(bytes, cover_end, field_offset) == stored
}

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
