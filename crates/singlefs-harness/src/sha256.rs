//! SHA-256（FIPS 180-4）本地实现：E142（第一个事务的干跑） 量 5 的区域摘要要能在仓外用 `sha256sum` 独立复算，
//! 所以摘要必须是标准 SHA-256——本模块只做摘要，不参与任何写路径。
//! 仓里另有一个 [`crate::fnv1a_64`]，那个只认「同一份内容」、不是标准摘要，仓外核不了。
//! 本地实现的理由与录制器同一条（`lib.rs` 的模块注释）：这个工作区不引第三方 crate。
//!
//! 名字里的 `sha256` 是 FIPS 180-4 给这个算法起的名字，不是我们的缩写。

use crate::hexadecimal::hexadecimal_text;

/// FIPS 180-4 第 4.2.2 节的 64 个轮常量（前 64 个素数立方根小数部分的前 32 位）。
const ROUND_CONSTANTS: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

/// FIPS 180-4 第 5.3.3 节的初始摘要（前 8 个素数平方根小数部分的前 32 位）。
const INITIAL_HASH_VALUES: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

/// 压缩函数一次吃 512 位。
const BLOCK_BYTES: usize = 64;
/// 尾部那个「消息位数」字段 64 位。
const MESSAGE_LENGTH_FIELD_BYTES: usize = 8;
/// 摘要 256 位。
pub const DIGEST_BYTES: usize = 32;

/// 一段字节的 SHA-256 摘要。
#[must_use]
pub fn sha256_digest(message: &[u8]) -> [u8; DIGEST_BYTES] {
    let mut hash_values = INITIAL_HASH_VALUES;
    let padded = padded_message(message);
    let (blocks, leftover) = padded.as_chunks::<BLOCK_BYTES>();
    assert!(
        leftover.is_empty(),
        "补白之后一定是整块：补白只会把长度补到 512 位的整数倍"
    );
    for block in blocks {
        compress_one_block(&mut hash_values, block);
    }
    let mut digest = [0u8; DIGEST_BYTES];
    for (word_index, word) in hash_values.iter().enumerate() {
        digest[word_index * 4..word_index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    digest
}

/// 一段字节的 SHA-256 摘要，小写十六进制（与 `sha256sum` 打出来的那 64 个字符逐字相同）。
#[must_use]
pub fn sha256_hexadecimal(message: &[u8]) -> String {
    hexadecimal_text(&sha256_digest(message))
}

/// FIPS 180-4 第 5.1.1 节的补白：接一个 0x80，补 0 到离块尾还差 8 字节，末尾写消息的位数（大端）。
fn padded_message(message: &[u8]) -> Vec<u8> {
    let message_length_in_bits = u64::try_from(message.len())
        .expect("消息字节数放得进 u64")
        .checked_mul(8)
        .expect("消息位数放得进 u64：FIPS 180-4 的上界是 2^64 位");
    let mut padded = Vec::with_capacity(message.len() + BLOCK_BYTES * 2);
    padded.extend_from_slice(message);
    padded.push(0x80);
    while !(padded.len() + MESSAGE_LENGTH_FIELD_BYTES).is_multiple_of(BLOCK_BYTES) {
        padded.push(0);
    }
    padded.extend_from_slice(&message_length_in_bits.to_be_bytes());
    padded
}

/// FIPS 180-4 第 6.2.2 节的一轮压缩。`working_variables` 的下标 0..=7 就是规范里的 a..h——
/// 那八个名字是规范定的单字母，本项目不另起名字，靠下标与这条注释对上。
fn compress_one_block(hash_values: &mut [u32; 8], block: &[u8; BLOCK_BYTES]) {
    let mut message_schedule = [0u32; 64];
    let (four_byte_words, leftover) = block.as_chunks::<4>();
    assert!(leftover.is_empty(), "512 位块正好切成 16 个 32 位字");
    for (word_index, four_bytes) in four_byte_words.iter().enumerate() {
        message_schedule[word_index] = u32::from_be_bytes(*four_bytes);
    }
    for word_index in 16..64 {
        let fifteen_words_back = message_schedule[word_index - 15];
        let two_words_back = message_schedule[word_index - 2];
        let small_sigma_zero = fifteen_words_back.rotate_right(7)
            ^ fifteen_words_back.rotate_right(18)
            ^ (fifteen_words_back >> 3);
        let small_sigma_one = two_words_back.rotate_right(17)
            ^ two_words_back.rotate_right(19)
            ^ (two_words_back >> 10);
        message_schedule[word_index] = message_schedule[word_index - 16]
            .wrapping_add(small_sigma_zero)
            .wrapping_add(message_schedule[word_index - 7])
            .wrapping_add(small_sigma_one);
    }
    let mut working_variables = *hash_values;
    for round in 0..64 {
        let big_sigma_one = working_variables[4].rotate_right(6)
            ^ working_variables[4].rotate_right(11)
            ^ working_variables[4].rotate_right(25);
        let choose = (working_variables[4] & working_variables[5])
            ^ (!working_variables[4] & working_variables[6]);
        let first_temporary = working_variables[7]
            .wrapping_add(big_sigma_one)
            .wrapping_add(choose)
            .wrapping_add(ROUND_CONSTANTS[round])
            .wrapping_add(message_schedule[round]);
        let big_sigma_zero = working_variables[0].rotate_right(2)
            ^ working_variables[0].rotate_right(13)
            ^ working_variables[0].rotate_right(22);
        let majority = (working_variables[0] & working_variables[1])
            ^ (working_variables[0] & working_variables[2])
            ^ (working_variables[1] & working_variables[2]);
        let second_temporary = big_sigma_zero.wrapping_add(majority);
        working_variables = [
            first_temporary.wrapping_add(second_temporary),
            working_variables[0],
            working_variables[1],
            working_variables[2],
            working_variables[3].wrapping_add(first_temporary),
            working_variables[4],
            working_variables[5],
            working_variables[6],
        ];
    }
    for (hash_value, working_variable) in hash_values.iter_mut().zip(working_variables) {
        *hash_value = hash_value.wrapping_add(working_variable);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 公开的测试向量（FIPS 180-2 附录 B 的三条 + 长消息那条），值与 `sha256sum` 打出来的逐字相同——
    /// 摘要算错时这里先红，量 5 的区域摘要就不会拿一个自洽却非标准的哈希去比。
    #[test]
    fn sha256_matches_the_published_test_vectors() {
        assert_eq!(
            sha256_hexadecimal(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hexadecimal(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hexadecimal(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
            "56 字节：补白之后正好跨两块，长度字段单独占一块"
        );
        assert_eq!(
            sha256_hexadecimal(&vec![b'a'; 1_000_000]),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0",
            "一百万个 a：长消息那条向量，位数字段超过 32 位"
        );
    }

    /// 区域摘要要认得出「差一位」：32 KiB 的单元里翻一位，摘要必须变。
    #[test]
    fn flipping_one_bit_of_a_data_unit_sized_message_changes_the_digest() {
        let message = vec![0x5au8; 32768];
        let mut flipped = message.clone();
        let flipped_index = 823 % flipped.len();
        flipped[flipped_index] ^= 0x01;
        assert_ne!(sha256_hexadecimal(&message), sha256_hexadecimal(&flipped));
    }
}
