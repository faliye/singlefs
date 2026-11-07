//! SHA-256（FIPS 180-4）：大值分段（`segmented_values`）的清单记整值的摘要，要与 Python 客户端 `hashlib.sha256(…).hexdigest()` 逐字相同。
//! 本工程是仓根工作区之外的独立工程，依赖只有 rocksdb 与 libc（Cargo.toml），不引第三方摘要包、也不依赖仓根的 crate，
//! 算法照 `crates/singlefs-verification/src/sha256.rs` 另写一份：整块直接从消息里压、只把尾巴拷出来补白，
//! 一百多 MiB 的值不为补白另拷一份。
//!
//! 名字里的 `sha256` 是 FIPS 180-4 给这个算法起的名字，不是我们的缩写。

use std::fmt::Write as _;

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
const DIGEST_BYTES: usize = 32;

/// 一段字节的 SHA-256 摘要，小写十六进制（与 `sha256sum`、Python 的 `hexdigest()` 打出来的那 64 个字符逐字相同）。
#[must_use]
pub fn sha256_hexadecimal(message: &[u8]) -> String {
    let mut hexadecimal = String::with_capacity(DIGEST_BYTES * 2);
    for byte in sha256_digest(message) {
        write!(hexadecimal, "{byte:02x}").expect("往 String 里写不会失败");
    }
    hexadecimal
}

/// 一段字节的 SHA-256 摘要。
fn sha256_digest(message: &[u8]) -> [u8; DIGEST_BYTES] {
    let mut hash_values = INITIAL_HASH_VALUES;
    let (whole_blocks, tail) = message.as_chunks::<BLOCK_BYTES>();
    for block in whole_blocks {
        compress_one_block(&mut hash_values, block);
    }
    let padded_tail = padded_tail_of_the_message(tail, message.len());
    let (tail_blocks, leftover) = padded_tail.as_chunks::<BLOCK_BYTES>();
    assert!(
        leftover.is_empty(),
        "补白之后一定是整块：补白只会把长度补到 512 位的整数倍"
    );
    for block in tail_blocks {
        compress_one_block(&mut hash_values, block);
    }
    let mut digest = [0u8; DIGEST_BYTES];
    for (word_index, word) in hash_values.iter().enumerate() {
        digest[word_index * 4..word_index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    digest
}

/// FIPS 180-4 第 5.1.1 节的补白，只对不足一块的尾巴做：接一个 0x80，补 0 到离块尾还差 8 字节，末尾写整条消息的位数（大端）。
fn padded_tail_of_the_message(tail: &[u8], message_length_in_bytes: usize) -> Vec<u8> {
    let message_length_in_bits = u64::try_from(message_length_in_bytes)
        .expect("消息字节数放得进 u64")
        .checked_mul(8)
        .expect("消息位数放得进 u64：FIPS 180-4 的上界是 2^64 位");
    let mut padded = Vec::with_capacity(BLOCK_BYTES * 2);
    padded.extend_from_slice(tail);
    padded.push(0x80);
    while !(padded.len() + MESSAGE_LENGTH_FIELD_BYTES).is_multiple_of(BLOCK_BYTES) {
        padded.push(0);
    }
    padded.extend_from_slice(&message_length_in_bits.to_be_bytes());
    padded
}

/// FIPS 180-4 第 6.2.2 节的一轮压缩。`working_variables` 的下标 0..=7 就是规范里的八个工作变量（规范用单字母起名，
/// 本项目不另起名字，靠下标与这条注释对上）。
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

    /// 公开的测试向量（FIPS 180-2 附录 B 的三条加长消息那条），值与 `sha256sum` 打出来的逐字相同：摘要算错时这里先红，
    /// 清单里记的就不会是一个自洽却与 Python 客户端对不上的哈希。56 字节那条补白之后跨两块，一百万个 a 那条走整块直接压的那一支（它是 8,000,000 位，不到 2^32，位数字段的高 32 位仍是 0；位数超过 32 位的长度字段另有一条用例）。
    /// 判别力（`crates/mutations.tsv`）：轮常量错一位，这条红。
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
            "一百万个 a：长消息那条向量，整块从消息里直接压；8,000,000 位，位数字段的高 32 位是 0"
        );
    }

    /// 位数超过 32 位的长度字段：消息 2^30 字节是 2^33 位，位数字段的高 32 位不为 0。不真分配那么大的消息，直接核补白函数（只吃尾巴与总长）末尾写的 8 个字节。
    /// 判别力：位数字段只写低 32 位（高 32 位丢成 0）时这条红。
    #[test]
    fn the_length_field_carries_message_bit_counts_beyond_32_bits() {
        let message_length_in_bytes: usize = 1 << 30;
        let padded = padded_tail_of_the_message(&[], message_length_in_bytes);
        assert_eq!(padded.len(), BLOCK_BYTES, "空尾巴补白之后正好一块");
        assert_eq!(padded[0], 0x80, "补白头一个字节是 0x80");
        let (_, length_field) = padded.split_at(BLOCK_BYTES - MESSAGE_LENGTH_FIELD_BYTES);
        assert_eq!(
            length_field,
            (1_u64 << 33).to_be_bytes(),
            "2^30 字节 = 2^33 位，大端写满 64 位"
        );
        assert_eq!(
            length_field[..4],
            [0, 0, 0, 2],
            "高 32 位是 2：这条要真测到超过 32 位的那一半"
        );
    }

    /// 正好整块的消息（64 字节）：尾巴是空的，补白单独占一整块；整块那一支与补白那一支各走一次。
    /// 判别力（`crates/mutations.tsv`）：跳过消息里的头一个整块，这条红。
    #[test]
    fn a_message_of_exactly_one_block_is_padded_into_a_block_of_its_own() {
        assert_eq!(
            sha256_hexadecimal(&[b'a'; 64]),
            "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb"
        );
    }
}
