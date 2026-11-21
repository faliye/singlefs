//! 分块喂入的 SHA-256。

use super::*;

/// 逐块喂的 SHA-256（FIPS 180-4），给操作流前缀算指纹；单测拿公开测试向量钉住。
#[derive(Clone, Debug)]
pub struct StreamingSha256 {
    hash_words: [u32; 8],
    pending_block: [u8; 64],
    pending_bytes: usize,
    total_bytes: u64,
}

const SHA256_INITIAL_HASH: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];

const SHA256_ROUND_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

impl StreamingSha256 {
    #[must_use]
    pub fn new() -> Self {
        Self { hash_words: SHA256_INITIAL_HASH, pending_block: [0; 64], pending_bytes: 0, total_bytes: 0 }
    }

    pub fn absorb_bytes(&mut self, bytes: &[u8]) {
        self.total_bytes += u64::try_from(bytes.len()).expect("一次喂的字节数装得进 u64");
        for byte in bytes {
            self.pending_block[self.pending_bytes] = *byte;
            self.pending_bytes += 1;
            if self.pending_bytes == 64 {
                let block = self.pending_block;
                self.compress(&block);
                self.pending_bytes = 0;
            }
        }
    }

    fn compress(&mut self, block: &[u8; 64]) {
        let mut schedule = [0u32; 64];
        for (word_index, word) in schedule.iter_mut().take(16).enumerate() {
            *word = u32::from_be_bytes(block[word_index * 4..word_index * 4 + 4].try_into().expect("切了 4 字节"));
        }
        for word_index in 16..64 {
            let sigma_zero = schedule[word_index - 15].rotate_right(7) ^ schedule[word_index - 15].rotate_right(18) ^ (schedule[word_index - 15] >> 3);
            let sigma_one = schedule[word_index - 2].rotate_right(17) ^ schedule[word_index - 2].rotate_right(19) ^ (schedule[word_index - 2] >> 10);
            schedule[word_index] = schedule[word_index - 16].wrapping_add(sigma_zero).wrapping_add(schedule[word_index - 7]).wrapping_add(sigma_one);
        }
        let mut registers = self.hash_words;
        for round in 0..64 {
            let big_sigma_one = registers[4].rotate_right(6) ^ registers[4].rotate_right(11) ^ registers[4].rotate_right(25);
            let choose = (registers[4] & registers[5]) ^ ((!registers[4]) & registers[6]);
            let temporary_one = registers[7]
                .wrapping_add(big_sigma_one)
                .wrapping_add(choose)
                .wrapping_add(SHA256_ROUND_CONSTANTS[round])
                .wrapping_add(schedule[round]);
            let big_sigma_zero = registers[0].rotate_right(2) ^ registers[0].rotate_right(13) ^ registers[0].rotate_right(22);
            let majority = (registers[0] & registers[1]) ^ (registers[0] & registers[2]) ^ (registers[1] & registers[2]);
            let temporary_two = big_sigma_zero.wrapping_add(majority);
            registers = [
                temporary_one.wrapping_add(temporary_two),
                registers[0],
                registers[1],
                registers[2],
                registers[3].wrapping_add(temporary_one),
                registers[4],
                registers[5],
                registers[6],
            ];
        }
        for (slot, value) in self.hash_words.iter_mut().zip(registers) {
            *slot = slot.wrapping_add(value);
        }
    }

    /// 收尾（补 0x80、零、位长），交回十六进制摘要。拿的是一份拷贝，原状态可以接着喂。
    #[must_use]
    pub fn hexadecimal_digest(&self) -> String {
        let mut finishing = self.clone();
        let bit_length = self.total_bytes * 8;
        finishing.absorb_bytes(&[0x80]);
        while finishing.pending_bytes != 56 {
            finishing.absorb_bytes(&[0]);
        }
        finishing.absorb_bytes(&bit_length.to_be_bytes());
        assert_eq!(finishing.pending_bytes, 0, "补齐之后正好一整块");
        finishing.hash_words.iter().fold(String::new(), |mut text, word| {
            write!(text, "{word:08x}").expect("写进 String 不会失败");
            text
        })
    }
}

impl Default for StreamingSha256 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "streaming_sha256_tests.rs"]
mod tests;
