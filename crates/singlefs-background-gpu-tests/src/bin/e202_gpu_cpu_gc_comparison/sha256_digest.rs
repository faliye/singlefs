//! SHA-256（FIPS 180-4）：录制流与读到的内容的摘要。

// ---- SHA-256（FIPS 180-4）：录制流与读到的内容的摘要。本包不依赖哈希库、派发不许改 Cargo.toml，在这里写一份，单测钉标准的已知答案 ----

const SHA256_ROUND_CONSTANTS: [u32; 64] = [
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
const SHA256_INITIAL_STATE: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];
const SHA256_BLOCK_BYTES: usize = 64;

/// 一份进行中的 SHA-256：状态、没凑满一块的尾巴、喂过的总字节数。
#[derive(Clone)]
pub(super) struct Sha256Digest {
    state: [u32; 8],
    pub(super) pending: Vec<u8>,
    bytes_fed: u64,
}

impl Sha256Digest {
    pub(super) fn new() -> Sha256Digest {
        Sha256Digest {
            state: SHA256_INITIAL_STATE,
            pending: Vec::with_capacity(SHA256_BLOCK_BYTES),
            bytes_fed: 0,
        }
    }

    pub(super) fn feed(&mut self, bytes: &[u8]) {
        self.bytes_fed += u64::try_from(bytes.len()).expect("一段字节数装得进 u64");
        let mut rest = bytes;
        if !self.pending.is_empty() {
            let wanted = SHA256_BLOCK_BYTES - self.pending.len();
            let taken = wanted.min(rest.len());
            self.pending.extend_from_slice(&rest[..taken]);
            rest = &rest[taken..];
            if self.pending.len() == SHA256_BLOCK_BYTES {
                let block: [u8; SHA256_BLOCK_BYTES] =
                    self.pending.as_slice().try_into().expect("凑满了一块");
                sha256_compress(&mut self.state, &block);
                self.pending.clear();
            }
        }
        let (whole_blocks, bytes_short_of_a_block) = rest.as_chunks::<SHA256_BLOCK_BYTES>();
        for block in whole_blocks {
            sha256_compress(&mut self.state, block);
        }
        self.pending.extend_from_slice(bytes_short_of_a_block);
    }

    pub(super) fn feed_u64(&mut self, value: u64) {
        self.feed(&value.to_le_bytes());
    }

    /// 收尾：补位、长度，交回 64 个十六进制字符。
    pub(super) fn finish_as_hex(mut self) -> String {
        let bit_length = self.bytes_fed.wrapping_mul(8);
        let mut padding = vec![0x80u8];
        let pending_after_the_marker = (self.pending.len() + 1) % SHA256_BLOCK_BYTES;
        let zero_bytes = (SHA256_BLOCK_BYTES + 56 - pending_after_the_marker) % SHA256_BLOCK_BYTES;
        padding.extend(std::iter::repeat_n(0u8, zero_bytes));
        padding.extend_from_slice(&bit_length.to_be_bytes());
        let bytes_fed_before_the_padding = self.bytes_fed;
        self.feed(&padding);
        self.bytes_fed = bytes_fed_before_the_padding;
        assert!(self.pending.is_empty(), "补位之后正好凑满整块");
        self.state
            .iter()
            .map(|word| format!("{word:08x}"))
            .collect()
    }
}

fn sha256_compress(state: &mut [u32; 8], block: &[u8; SHA256_BLOCK_BYTES]) {
    let mut schedule = [0u32; 64];
    let (words_of_the_block, bytes_short_of_a_word) = block.as_chunks::<4>();
    assert!(
        bytes_short_of_a_word.is_empty(),
        "一块 64 字节正好是 16 个四字节的字"
    );
    for (word_index, word) in words_of_the_block.iter().enumerate() {
        schedule[word_index] = u32::from_be_bytes(*word);
    }
    for round in 16..64 {
        let word_fifteen_rounds_earlier = schedule[round - 15];
        let word_two_rounds_earlier = schedule[round - 2];
        let small_sigma_of_the_older_word = word_fifteen_rounds_earlier.rotate_right(7)
            ^ word_fifteen_rounds_earlier.rotate_right(18)
            ^ (word_fifteen_rounds_earlier >> 3);
        let small_sigma_of_the_newer_word = word_two_rounds_earlier.rotate_right(17)
            ^ word_two_rounds_earlier.rotate_right(19)
            ^ (word_two_rounds_earlier >> 10);
        schedule[round] = schedule[round - 16]
            .wrapping_add(small_sigma_of_the_older_word)
            .wrapping_add(schedule[round - 7])
            .wrapping_add(small_sigma_of_the_newer_word);
    }
    // 八个工作字是两条移位链：每一轮只算出链头的新值，其余各字是链头一轮、两轮、三轮之前的值。
    let mut working = *state;
    for round in 0..64 {
        let [head_word, head_word_one_round_ago, head_word_two_rounds_ago, head_word_three_rounds_ago, middle_word, middle_word_one_round_ago, middle_word_two_rounds_ago, middle_word_three_rounds_ago] =
            working;
        let big_sigma_of_the_middle_word = middle_word.rotate_right(6)
            ^ middle_word.rotate_right(11)
            ^ middle_word.rotate_right(25);
        let choice =
            (middle_word & middle_word_one_round_ago) ^ (!middle_word & middle_word_two_rounds_ago);
        let mixed_into_both_chains = middle_word_three_rounds_ago
            .wrapping_add(big_sigma_of_the_middle_word)
            .wrapping_add(choice)
            .wrapping_add(SHA256_ROUND_CONSTANTS[round])
            .wrapping_add(schedule[round]);
        let big_sigma_of_the_head_word =
            head_word.rotate_right(2) ^ head_word.rotate_right(13) ^ head_word.rotate_right(22);
        let majority = (head_word & head_word_one_round_ago)
            ^ (head_word & head_word_two_rounds_ago)
            ^ (head_word_one_round_ago & head_word_two_rounds_ago);
        let mixed_into_the_head_chain = big_sigma_of_the_head_word.wrapping_add(majority);
        working = [
            mixed_into_both_chains.wrapping_add(mixed_into_the_head_chain),
            head_word,
            head_word_one_round_ago,
            head_word_two_rounds_ago,
            head_word_three_rounds_ago.wrapping_add(mixed_into_both_chains),
            middle_word,
            middle_word_one_round_ago,
            middle_word_two_rounds_ago,
        ];
    }
    for (state_word, working_word) in state.iter_mut().zip(working) {
        *state_word = state_word.wrapping_add(working_word);
    }
}

#[cfg(test)]
mod tests;
