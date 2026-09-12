//! E61：反向链 hash 算法的均匀性 —— D23（journal 的角色与格式） 已定项 11 那个「用什么函数」。
//!
//! D23（journal 的角色与格式） 已定项 8 定了宽度 32 位、已定项 10 定了覆盖范围（前一条记录的
//! 整个记录头，`header_csum` 那 32 字节不进输入）。缺的只是函数。
//!
//! 核心手法：**CRC 是 GF(2) 上的线性映射**，所以「差异被限制在一个 32 比特窗口内时
//! 碰撞数是多少」可以精确算，不必枚举 2³² 个差异——
//! 碰撞数恰好为 0 ⟺ 该窗口 32 个基向量的像线性无关（秩 = 32）。
//! 密码学 hash 不线性，这条路走不通，只能抽样——**这本身就是一个结论**。
//!
//! ⚠️ 三个 hash 各自独立实现，不复用 E40（密文校验和位宽） 的那一份：
//! 复用等于让被测对象与它的正确性证据共享代码（C48（多条校验路径共用同一个前提））。

use std::collections::HashMap;

// ───────────────────────── 被测的三个函数，各自独立实现 ─────────────────────────

/// 甲：完整 32 位 CRC32C（Castagnoli）。反射式，生成多项式 0x1EDC6F41 反射后为 0x82F63B78。
fn crc32c(data: &[u8]) -> u32 {
    let mut register: u32 = 0xFFFF_FFFF;
    for &byte in data {
        register ^= byte as u32;
        for _ in 0..8 {
            register = if register & 1 != 0 { (register >> 1) ^ 0x82F6_3B78 } else { register >> 1 };
        }
    }
    !register
}

/// 丙（阳性对照）：CRC-64/XZ，取低 32 位。反射多项式 0xC96C5795D7870F42。
fn cyclic_redundancy_check_64_xz(data: &[u8]) -> u64 {
    let mut register: u64 = 0xFFFF_FFFF_FFFF_FFFF;
    for &byte in data {
        register ^= byte as u64;
        for _ in 0..8 {
            register = if register & 1 != 0 { (register >> 1) ^ 0xC96C_5795_D787_0F42 } else { register >> 1 };
        }
    }
    !register
}
fn cyclic_redundancy_check_64_truncated_to_32_bits(data: &[u8]) -> u32 {
    cyclic_redundancy_check_64_xz(data) as u32
}

/// 乙：SHA-256，取前 4 字节（大端）。从头实现，由 FIPS 180-4 的测试向量钉死。
fn sha256(data: &[u8]) -> [u8; 32] {
    const ROUND_CONSTANTS: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut hash_words: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let mut padded_message = data.to_vec();
    let message_bit_length = (data.len() as u64) * 8;
    padded_message.push(0x80);
    while padded_message.len() % 64 != 56 {
        padded_message.push(0);
    }
    padded_message.extend_from_slice(&message_bit_length.to_be_bytes());

    for chunk in padded_message.chunks(64) {
        let mut message_schedule = [0u32; 64];
        for word_index in 0..16 {
            message_schedule[word_index] = u32::from_be_bytes([chunk[4 * word_index], chunk[4 * word_index + 1], chunk[4 * word_index + 2], chunk[4 * word_index + 3]]);
        }
        for word_index in 16..64 {
            let small_sigma_zero = message_schedule[word_index - 15].rotate_right(7) ^ message_schedule[word_index - 15].rotate_right(18) ^ (message_schedule[word_index - 15] >> 3);
            let small_sigma_one = message_schedule[word_index - 2].rotate_right(17) ^ message_schedule[word_index - 2].rotate_right(19) ^ (message_schedule[word_index - 2] >> 10);
            message_schedule[word_index] = message_schedule[word_index - 16]
                .wrapping_add(small_sigma_zero)
                .wrapping_add(message_schedule[word_index - 7])
                .wrapping_add(small_sigma_one);
        }
        let (mut state_word_0, mut state_word_1, mut state_word_2, mut state_word_3) = (hash_words[0], hash_words[1], hash_words[2], hash_words[3]);
        let (mut state_word_4, mut state_word_5, mut state_word_6, mut state_word_7) = (hash_words[4], hash_words[5], hash_words[6], hash_words[7]);
        for round_index in 0..64 {
            let big_sigma_one = state_word_4.rotate_right(6) ^ state_word_4.rotate_right(11) ^ state_word_4.rotate_right(25);
            let choose = (state_word_4 & state_word_5) ^ ((!state_word_4) & state_word_6);
            let temporary_one = state_word_7
                .wrapping_add(big_sigma_one)
                .wrapping_add(choose)
                .wrapping_add(ROUND_CONSTANTS[round_index])
                .wrapping_add(message_schedule[round_index]);
            let big_sigma_zero = state_word_0.rotate_right(2) ^ state_word_0.rotate_right(13) ^ state_word_0.rotate_right(22);
            let majority = (state_word_0 & state_word_1) ^ (state_word_0 & state_word_2) ^ (state_word_1 & state_word_2);
            let temporary_two = big_sigma_zero.wrapping_add(majority);
            state_word_7 = state_word_6; state_word_6 = state_word_5; state_word_5 = state_word_4; state_word_4 = state_word_3.wrapping_add(temporary_one);
            state_word_3 = state_word_2; state_word_2 = state_word_1; state_word_1 = state_word_0; state_word_0 = temporary_one.wrapping_add(temporary_two);
        }
        for (state_index, added_word) in [state_word_0, state_word_1, state_word_2, state_word_3, state_word_4, state_word_5, state_word_6, state_word_7].into_iter().enumerate() {
            hash_words[state_index] = hash_words[state_index].wrapping_add(added_word);
        }
    }
    let mut digest = [0u8; 32];
    for output_word_index in 0..8 {
        digest[4 * output_word_index..4 * output_word_index + 4].copy_from_slice(&hash_words[output_word_index].to_be_bytes());
    }
    digest
}
fn sha256_truncated_to_32_bits(data: &[u8]) -> u32 {
    let digest = sha256(data);
    u32::from_be_bytes([digest[0], digest[1], digest[2], digest[3]])
}

// ───────────────────────── 记录头模型（E23（journal 几何） 的 11 个字段）─────────────────────────

/// `header_csum` 那 32 字节不进 hash 输入（D23（journal 的角色与格式） 已定项 10 的实现纪律：当零）。
const HEADER_CHECKSUM_BYTES: usize = 32;
/// ⚠️ 这三档是**跑这一轮时**按当时的字段表建的模（86 = 含 `tail_lsn` 那 8 字节的版本）。
/// 2026-08-31 `tail_lsn` 随 D23（journal 的角色与格式） 已定项 3 去掉 ⇒ 现行是 78 / 87 / 91。
/// 本文件连同它的产物是那一轮的记录，**不跟着改**——改了产物就不再对应它的输入。
/// E61（反向链 hash 算法的均匀性） 的量二结论已作废，见该实验正文开头。
const HEADER_LENGTHS_IN_BYTES: [usize; 3] = [86, 95, 99];

/// 造一个形态真实的记录头：可变字段填满，`header_csum` 段清零。
fn make_header(header_length_in_bytes: usize, jsn: u64, epoch: u32, checkpoint_txg: u64) -> Vec<u8> {
    let mut header_bytes = vec![0u8; header_length_in_bytes];
    header_bytes[0..4].copy_from_slice(&0x6A_53_4E_31u32.to_be_bytes()); // magic
    header_bytes[4..6].copy_from_slice(&1u16.to_be_bytes());             // version/type
    header_bytes[6] = 0;                                                 // algo_type
    header_bytes[7] = 0;                                                 // 对齐填充
    header_bytes[8..12].copy_from_slice(&(header_length_in_bytes as u32).to_be_bytes()); // record_length
    header_bytes[12..16].copy_from_slice(&7u32.to_be_bytes());            // named_count
    header_bytes[16..20].copy_from_slice(&epoch.to_be_bytes());           // jsn 高段：实例代号 32 位
    header_bytes[20..26].copy_from_slice(&jsn.to_be_bytes()[2..]);        // jsn 低段：计数器 48 位
    header_bytes[26..34].copy_from_slice(&checkpoint_txg.to_be_bytes());             // checkpoint_txg
    header_bytes[34..42].copy_from_slice(&(jsn ^ 0x5A5A).to_be_bytes());  // tail_lsn
    for nonce_byte_index in 42..54 {
        header_bytes[nonce_byte_index] = (jsn as u8).wrapping_add(nonce_byte_index as u8);              // nonce 12 字节
    }
    // 54..54+32 是 header_csum，按已定项 10 的纪律留零，不进输入
    header_bytes
}
/// 可变区（差异只可能落在这里）的比特数。
fn variable_bits(header_length_in_bytes: usize) -> usize {
    (header_length_in_bytes - HEADER_CHECKSUM_BYTES) * 8
}

// ───────────────────────── GF(2) 秩 ─────────────────────────

/// 32 个像向量在 GF(2) 上的秩。秩 = 32 ⟺ 该窗口内 2³² 个差异一个都不碰撞。
fn binary_field_rank(mut image_vectors: Vec<u32>) -> usize {
    let mut rank = 0;
    for bit_position in (0..32).rev() {
        if let Some(pivot_row) = (rank..image_vectors.len()).find(|&row_index| image_vectors[row_index] >> bit_position & 1 == 1) {
            image_vectors.swap(rank, pivot_row);
            let pivot = image_vectors[rank];
            for row_index in 0..image_vectors.len() {
                if row_index != rank && (image_vectors[row_index] >> bit_position & 1 == 1) {
                    image_vectors[row_index] ^= pivot;
                }
            }
            rank += 1;
        }
    }
    rank
}

/// 秩 r 的窗口里，**非零差异**中会碰撞的个数 = 2^(32−r) − 1。满秩时恰好 0。
/// ⚠️ 不是核的大小——核含零向量，而零差异是同源比较，按 E51（反向链的碰撞机会有多少次）
/// 判据 1 根本不构成碰撞机会。
fn colliding_differences(rank: usize) -> u64 {
    (1u64 << (32 - rank as u32)) - 1
}

/// 一个 32 比特窗口的像基：`f(base XOR e_i) XOR f(base)`。
/// 对线性的 f（CRC 及其截断）这与 base 无关——`base_independent` 那个单测钉的就是这一条。
fn window_image<HashFunction: Fn(&[u8]) -> u32>(hash_function: &HashFunction, base: &[u8], window_start_bit: usize) -> Vec<u32> {
    let base_hash = hash_function(base);
    (0..32)
        .map(|bit_in_window| {
            let mut flipped_input = base.to_vec();
            let flipped_bit = window_start_bit + bit_in_window;
            flipped_input[flipped_bit / 8] ^= 1u8 << (flipped_bit % 8);
            hash_function(&flipped_input) ^ base_hash
        })
        .collect()
}

/// 扫遍可变区里所有 32 比特窗口，报「秩满的窗口数 / 总窗口数」与最差秩。
fn rank_spectrum<HashFunction: Fn(&[u8]) -> u32>(hash_function: &HashFunction, header_length_in_bytes: usize) -> (usize, usize, usize, u64) {
    let base = make_header(header_length_in_bytes, 0x0000_1234_5678, 0xDEAD_BEEF, 0x00AA);
    let variable_bit_count = variable_bits(header_length_in_bytes);
    let (mut full_rank_window_count, mut total_window_count, mut worst_rank) = (0usize, 0usize, 32usize);
    let mut worst_collisions: u64 = 0;
    for window_start_bit in 0..=(variable_bit_count - 32) {
        let rank = binary_field_rank(window_image(hash_function, &base, window_start_bit));
        total_window_count += 1;
        if rank == 32 { full_rank_window_count += 1; } else if rank < worst_rank {
            worst_rank = rank;
            worst_collisions = colliding_differences(rank);
        }
    }
    (full_rank_window_count, total_window_count, worst_rank, worst_collisions)
}

// ───────────────────────── 抽样（非线性臂只能这么问）─────────────────────────

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut mixed = *state;
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    mixed ^ (mixed >> 31)
}

/// 生日法：N 个互不相同的记录头，数碰撞。期望 = N(N−1)/2 / 2³²。
fn birthday<HashFunction: Fn(&[u8]) -> u32>(hash_function: &HashFunction, header_length_in_bytes: usize, header_count: usize, seed: u64) -> (u64, f64) {
    let mut splitmix_state = seed;
    let mut occurrences_by_hash: HashMap<u32, u32> = HashMap::with_capacity(header_count * 2);
    let mut collision_count: u64 = 0;
    for _ in 0..header_count {
        let header = make_header(
            header_length_in_bytes,
            splitmix64(&mut splitmix_state) & 0xFFFF_FFFF_FFFF,
            splitmix64(&mut splitmix_state) as u32,
            splitmix64(&mut splitmix_state),
        );
        let hash_value = hash_function(&header);
        let previous_occurrences = occurrences_by_hash.entry(hash_value).or_insert(0);
        collision_count += *previous_occurrences as u64;
        *previous_occurrences += 1;
    }
    let expected_collisions = (header_count as f64) * ((header_count - 1) as f64) / 2.0 / 4_294_967_296.0;
    (collision_count, expected_collisions)
}

// ───────────────────────── 输出 ─────────────────────────

struct Emitter(u32);
impl Emitter {
    fn emit(&mut self, line: String) {
        self.0 += 1;
        println!("E7RESULT {line}");
    }
}

fn main() {
    let mut emitter = Emitter(0);
    emitter.emit(format!(
        "name=config csum_bytes={HEADER_CHECKSUM_BYTES} hdr_lens=86/95/99 birthday_n=1048576"
    ));

    // ── 量一：秩谱。甲与丙线性，可精确判；乙不线性，不适用 ──
    for &header_length in &HEADER_LENGTHS_IN_BYTES {
        for (arm, name) in [
            (&crc32c as &dyn Fn(&[u8]) -> u32, "jia_crc32c"),
            (&cyclic_redundancy_check_64_truncated_to_32_bits as &dyn Fn(&[u8]) -> u32, "bing_crc64_trunc32"),
        ] {
            let hash_function = |input_bytes: &[u8]| arm(input_bytes);
            let (full_rank_window_count, total_window_count, worst_rank, worst_window_collisions) = rank_spectrum(&hash_function, header_length);
            emitter.emit(format!(
                "name=rank_spectrum arm={name} hdr_len={header_length} windows_full_rank={full_rank_window_count} windows_total={total_window_count} \
                 worst_rank={worst_rank} worst_window_collisions={worst_window_collisions}"
            ));
        }
    }

    // ── 量二：结构性差异——只有 jsn 的实例代号那 4 字节不同（两条时间线的实际形态）──
    for &header_length in &HEADER_LENGTHS_IN_BYTES {
        for (arm, name) in [
            (&crc32c as &dyn Fn(&[u8]) -> u32, "jia_crc32c"),
            (&cyclic_redundancy_check_64_truncated_to_32_bits as &dyn Fn(&[u8]) -> u32, "bing_crc64_trunc32"),
        ] {
            let hash_function = |input_bytes: &[u8]| arm(input_bytes);
            let base = make_header(header_length, 0x0000_1234_5678, 0xDEAD_BEEF, 0x00AA);
            let rank = binary_field_rank(window_image(&hash_function, &base, 16 * 8)); // 实例代号在字节 16..20
            emitter.emit(format!(
                "name=epoch_window arm={name} hdr_len={header_length} rank={rank} collisions={}",
                colliding_differences(rank)
            ));
        }
    }

    // ── 量三：抽样。三条臂都跑，乙是唯一只能靠它的 ──
    for &header_length in &HEADER_LENGTHS_IN_BYTES {
        for (arm, name) in [
            (&crc32c as &dyn Fn(&[u8]) -> u32, "jia_crc32c"),
            (&sha256_truncated_to_32_bits as &dyn Fn(&[u8]) -> u32, "yi_sha256_trunc32"),
            (&cyclic_redundancy_check_64_truncated_to_32_bits as &dyn Fn(&[u8]) -> u32, "bing_crc64_trunc32"),
        ] {
            let hash_function = |input_bytes: &[u8]| arm(input_bytes);
            let (collision_count, expected_collisions) = birthday(&hash_function, header_length, 1 << 20, 0x1234_5678_9ABC_DEF0);
            emitter.emit(format!(
                "name=birthday arm={name} hdr_len={header_length} collisions={collision_count} expect={expected_collisions:.2} ratio={:.3}",
                collision_count as f64 / expected_collisions
            ));
        }
    }

    // ── 量四：同源必须恒等（E51（反向链的碰撞机会有多少次） 判据 1 同型）──
    let header = make_header(86, 42, 7, 9);
    emitter.emit(format!(
        "name=same_source crc32c_equal={} sha_equal={} crc64_equal={}",
        crc32c(&header) == crc32c(&header),
        sha256_truncated_to_32_bits(&header) == sha256_truncated_to_32_bits(&header),
        cyclic_redundancy_check_64_truncated_to_32_bits(&header) == cyclic_redundancy_check_64_truncated_to_32_bits(&header)
    ));

    println!("E7RESULT name=done emitted={}", emitter.0 + 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 判据 3 的绝对值断言之一：CRC32C 的标准测试向量。
    /// **独立来源**（Castagnoli，与被测代码无关）：CRC-32C("123456789") = 0xE3069283。
    #[test]
    fn crc32c_matches_the_published_vector() {
        assert_eq!(crc32c(b"123456789"), 0xE306_9283);
        assert_eq!(crc32c(b""), 0x0000_0000);
    }

    /// 判据 3 之二：CRC-64/XZ 的标准测试向量 0x995DC9BBDF1939FA。
    #[test]
    fn cyclic_redundancy_check_64_matches_the_published_vector() {
        assert_eq!(cyclic_redundancy_check_64_xz(b"123456789"), 0x995D_C9BB_DF19_39FA);
        assert_eq!(cyclic_redundancy_check_64_truncated_to_32_bits(b"123456789"), 0xDF19_39FA);
    }

    /// 判据 3 之三：SHA-256 的 FIPS 180-4 测试向量。
    #[test]
    fn sha256_matches_the_published_vector() {
        let digest = sha256(b"abc");
        assert_eq!(&digest[..4], &[0xBA, 0x78, 0x16, 0xBF]);
        assert_eq!(sha256_truncated_to_32_bits(b"abc"), 0xBA78_16BF);
        // 空串向量，覆盖 padding 的另一条路径
        assert_eq!(sha256_truncated_to_32_bits(b""), 0xE3B0_C442);
    }

    /// 秩判据成立的前提：CRC 的差分与 base 无关（线性）。不成立则整个量一无意义。
    #[test]
    fn cyclic_redundancy_check_difference_is_base_independent() {
        let first_header = make_header(86, 1, 1, 1);
        let second_header = make_header(86, 999, 12345, 777);
        for window_start_bit in [0usize, 16 * 8, 40 * 8] {
            assert_eq!(
                window_image(&crc32c, &first_header, window_start_bit),
                window_image(&crc32c, &second_header, window_start_bit),
                "CRC32C 的差分依赖了 base，说明它不是线性映射"
            );
        }
    }

    /// **阳性对照本身要先证明它测得出差别**：SHA-256 的差分必须依赖 base，
    /// 否则「乙不能用秩判据」这个结论是空的。
    #[test]
    fn sha256_difference_is_not_base_independent() {
        let first_header = make_header(86, 1, 1, 1);
        let second_header = make_header(86, 999, 12345, 777);
        assert_ne!(window_image(&sha256_truncated_to_32_bits, &first_header, 0), window_image(&sha256_truncated_to_32_bits, &second_header, 0));
    }

    /// 报数口径的绝对值断言：满秩 ⇒ 0 个碰撞，不是 1 个（1 是核里那个零向量，属同源比较）。
    #[test]
    fn colliding_differences_excludes_the_zero_vector() {
        assert_eq!(colliding_differences(32), 0);
        assert_eq!(colliding_differences(31), 1);
        assert_eq!(colliding_differences(28), 15);
    }

    /// GF(2) 求秩本身的判别力：满秩、缺一秩、零矩阵三档都要判对。
    #[test]
    fn binary_field_rank_has_discrimination() {
        let full: Vec<u32> = (0..32).map(|bit_index| 1u32 << bit_index).collect();
        assert_eq!(binary_field_rank(full), 32);
        let mut deficient: Vec<u32> = (0..32).map(|bit_index| 1u32 << bit_index).collect();
        deficient[31] = deficient[0] ^ deficient[1]; // 制造一条线性相关
        assert_eq!(binary_field_rank(deficient), 31);
        assert_eq!(binary_field_rank(vec![0u32; 32]), 0);
        // 消元不做会漏秩：{0b11, 0b10} 的真秩是 2，只按当前位找主元会数成 1
        // （变异审计实测：上面三档在禁掉消元后照样全对——单位向量彼此不重叠，
        //  消元从没被走到；这一格才是消元的判别用例）。
        assert_eq!(binary_field_rank(vec![0b11, 0b10]), 2);
    }

    /// 记录头模型的绝对值断言：三档长度、可变区比特数、csum 段必须为零。
    #[test]
    fn header_model_is_pinned() {
        assert_eq!(HEADER_LENGTHS_IN_BYTES, [86, 95, 99]);
        assert_eq!(variable_bits(86), 54 * 8);
        assert_eq!(variable_bits(99), 67 * 8);
        let header = make_header(86, 42, 7, 9);
        assert_eq!(header.len(), 86);
        assert!(header[54..86].iter().all(|&byte| byte == 0), "header_csum 段必须留零，它不进 hash 输入");
    }

    /// 同源比较不构成碰撞机会：同一份输入必须构造性相等。
    #[test]
    fn same_source_is_constructively_equal() {
        let header = make_header(95, 12345, 999, 42);
        assert_eq!(crc32c(&header), crc32c(&header));
        assert_eq!(sha256_truncated_to_32_bits(&header), sha256_truncated_to_32_bits(&header));
    }
}
