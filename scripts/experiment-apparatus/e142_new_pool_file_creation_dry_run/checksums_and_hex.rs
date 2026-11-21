//! CRC32C 与宽校验和、SHA-256、十六进制、结果行的解析与字节差异摘要。

use e7_index_bench::usize_from_u64;
use super::*;

// ───────────────────────── 校验和：CRC32C（D19 已定项 2 / D23 已定项 11）与 32 字节摘要（算法未定，装置取 SHA-256） ─────────────────────────

const CASTAGNOLI_POLYNOMIAL_REFLECTED: u32 = 0x82F6_3B78;

fn castagnoli_tables() -> &'static [[u32; 256]; 8] {
    static TABLES: std::sync::OnceLock<[[u32; 256]; 8]> = std::sync::OnceLock::new();
    TABLES.get_or_init(|| {
        let mut tables = [[0u32; 256]; 8];
        for byte_value in 0..256u32 {
            let mut remainder = byte_value;
            for _ in 0..8 {
                remainder = if remainder & 1 == 1 { (remainder >> 1) ^ CASTAGNOLI_POLYNOMIAL_REFLECTED } else { remainder >> 1 };
            }
            tables[0][byte_value as usize] = remainder;
        }
        for byte_value in 0..256usize {
            let mut remainder = tables[0][byte_value];
            for table_index in 1..8 {
                remainder = tables[0][(remainder & 0xff) as usize] ^ (remainder >> 8);
                tables[table_index][byte_value] = remainder;
            }
        }
        tables
    })
}

/// 完整 32 位 CRC32C（Castagnoli，反射，初值与输出异或都是全 1），slicing-by-8。
pub(super) fn castagnoli_crc32(bytes: &[u8]) -> u32 {
    let tables = castagnoli_tables();
    let mut remainder = !0u32;
    let (chunks, chunk_remainder) = bytes.as_chunks::<8>();
    for chunk in chunks {
        let low = u32::from_le_bytes(chunk[..4].try_into().expect("切了 4 字节")) ^ remainder;
        let high = u32::from_le_bytes(chunk[4..].try_into().expect("切了 4 字节"));
        remainder = tables[7][(low & 0xff) as usize]
            ^ tables[6][((low >> 8) & 0xff) as usize]
            ^ tables[5][((low >> 16) & 0xff) as usize]
            ^ tables[4][(low >> 24) as usize]
            ^ tables[3][(high & 0xff) as usize]
            ^ tables[2][((high >> 8) & 0xff) as usize]
            ^ tables[1][((high >> 16) & 0xff) as usize]
            ^ tables[0][(high >> 24) as usize];
    }
    for &byte in chunk_remainder {
        remainder = tables[0][((remainder ^ u32::from(byte)) & 0xff) as usize] ^ (remainder >> 8);
    }
    !remainder
}

/// 「校验和字段自身按 0 参与」（I-2.4）：把 `[field_offset, field_offset + 32)` 清零后对 `[0, cover_end)` 求校验和。
/// D18（块里携带什么信息） 已定项 17：32 字节的校验和字段里放 CRC32C 4 字节 + 28 字节零，
/// 根记录自证校验和、系统配置整槽校验和、journal `header_csum` 同口径。
pub(super) fn wide_checksum_with_field_zeroed(bytes: &[u8], cover_end: usize, field_offset: usize) -> [u8; 32] {
    let mut covered = bytes[..cover_end].to_vec();
    covered[field_offset..field_offset + usize_from_u64(WIDE_CHECKSUM_BYTES)].fill(0);
    let mut field = [0u8; 32];
    field[..WIDE_CHECKSUM_CRC_BYTES].copy_from_slice(&castagnoli_crc32(&covered).to_le_bytes());
    field
}

// ───────────────────────── SHA-256（E142 第十一次跑量 5：与 crates/ 侧 `new_pool_file_creation_region_bytes` 的 `sha256=` 对齐）─────────────────────────
// 标准 FIPS 180-4 实现，不加新依赖（这个包只依赖 aes-gcm / chacha20poly1305，都不带现成的 SHA-256）。
// 64 个圆常量与 8 个初始哈希值都是规范里那两组固定质数算出来的值，单测钉了 "" 与 "abc" 两个标准测试向量。
const SHA256_INITIAL_HASH: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
const SHA256_ROUND_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
    0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
    0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

pub(super) fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut hash = SHA256_INITIAL_HASH;
    let bit_length = (bytes.len() as u64) * 8;
    let mut message = bytes.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_length.to_be_bytes());
    for chunk in message.as_chunks::<64>().0 {
        let mut schedule = [0u32; 64];
        for (index, word) in schedule.iter_mut().take(16).enumerate() {
            *word = u32::from_be_bytes(chunk[index * 4..index * 4 + 4].try_into().expect("切了 4 字节"));
        }
        for index in 16..64 {
            let sigma0 = schedule[index - 15].rotate_right(7) ^ schedule[index - 15].rotate_right(18) ^ (schedule[index - 15] >> 3);
            let sigma1 = schedule[index - 2].rotate_right(17) ^ schedule[index - 2].rotate_right(19) ^ (schedule[index - 2] >> 10);
            schedule[index] = schedule[index - 16].wrapping_add(sigma0).wrapping_add(schedule[index - 7]).wrapping_add(sigma1);
        }
        let [mut register_first, mut register_second, mut register_third, mut register_fourth, mut register_fifth, mut register_sixth, mut register_seventh, mut register_eighth] = hash;
        for index in 0..64 {
            let big_sigma1 = register_fifth.rotate_right(6) ^ register_fifth.rotate_right(11) ^ register_fifth.rotate_right(25);
            let choose = (register_fifth & register_sixth) ^ ((!register_fifth) & register_seventh);
            let temporary_one = register_eighth.wrapping_add(big_sigma1).wrapping_add(choose).wrapping_add(SHA256_ROUND_CONSTANTS[index]).wrapping_add(schedule[index]);
            let big_sigma0 = register_first.rotate_right(2) ^ register_first.rotate_right(13) ^ register_first.rotate_right(22);
            let majority = (register_first & register_second) ^ (register_first & register_third) ^ (register_second & register_third);
            let temporary_two = big_sigma0.wrapping_add(majority);
            register_eighth = register_seventh;
            register_seventh = register_sixth;
            register_sixth = register_fifth;
            register_fifth = register_fourth.wrapping_add(temporary_one);
            register_fourth = register_third;
            register_third = register_second;
            register_second = register_first;
            register_first = temporary_one.wrapping_add(temporary_two);
        }
        for (slot, value) in hash.iter_mut().zip([register_first, register_second, register_third, register_fourth, register_fifth, register_sixth, register_seventh, register_eighth]) {
            *slot = slot.wrapping_add(value);
        }
    }
    let mut digest = [0u8; 32];
    for (index, word) in hash.iter().enumerate() {
        digest[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    digest
}

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    hex_bytes(&sha256(bytes))
}

pub(super) fn hex_decode(hex: &str) -> Vec<u8> {
    (0..hex.len() / 2).map(|index| u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).unwrap_or(0)).collect()
}

/// 一行 `E7RESULT name=... key=value key=value...` 拆成 key→value；量 5 用它读 crates 侧的产出（自己格式自己拆，不新加依赖）。
pub(super) fn parse_result_line(line: &str) -> std::collections::BTreeMap<String, String> {
    line.split_whitespace().filter_map(|token| token.split_once('=')).map(|(key, value)| (key.to_string(), value.to_string())).collect()
}

/// 两段等长字节比出第一个不同的下标与不同字节总数；长度不等时直接报「整段都算」。
pub(super) fn byte_diff_summary(left_bytes: &[u8], right_bytes: &[u8]) -> (Option<u64>, Option<u64>) {
    if left_bytes.len() != right_bytes.len() {
        return (Some(0), Some(left_bytes.len().max(right_bytes.len()) as u64));
    }
    let mut first_diff = None;
    let mut mismatch_count = 0u64;
    for (index, (left, right)) in left_bytes.iter().zip(right_bytes.iter()).enumerate() {
        if left != right {
            if first_diff.is_none() {
                first_diff = Some(index as u64);
            }
            mismatch_count += 1;
        }
    }
    (first_diff, Some(mismatch_count))
}

#[cfg(test)]
#[path = "checksums_and_hex_tests.rs"]
mod tests;
