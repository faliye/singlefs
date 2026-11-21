//! 128 位确定性散列（登记 5.1）：内容、调用、键序列与键集合的摘要。

// ───────────────────────────── 128 位确定性散列（5.1） ─────────────────────────────

/// 128 位摘要：内容与键都用它。确定性（与线程数、跑的次数无关），std 之外自己写。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct Digest128(pub(super) u128);

const DIGEST_MULTIPLIER_LOW: u64 = 0x9E37_79B9_7F4A_7C15;
const DIGEST_MULTIPLIER_HIGH: u64 = 0xC2B2_AE3D_27D4_EB4F;
const DIGEST_SEED_LOW: u64 = 0x243F_6A88_85A3_08D3;
const DIGEST_SEED_HIGH: u64 = 0x1319_8A2E_0370_7344;

/// 64 位的末轮搅拌（splitmix64 的那一步）。
fn avalanche_word(word: u64) -> u64 {
    let mut mixed = word ^ (word >> 30);
    mixed = mixed.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed ^= mixed >> 27;
    mixed = mixed.wrapping_mul(0x94D0_49BB_1331_11EB);
    mixed ^ (mixed >> 31)
}

/// 逐个吃 64 位字、最后出 128 位摘要。`domain_tag` 把不同种类的输入分开（同样的字按不同种类喂出不同摘要）。
pub(super) struct DigestBuilder {
    low: u64,
    high: u64,
    words_fed: u64,
}

impl DigestBuilder {
    pub(super) fn new(domain_tag: u64) -> Self {
        Self {
            low: DIGEST_SEED_LOW ^ domain_tag,
            high: DIGEST_SEED_HIGH ^ domain_tag.rotate_left(32),
            words_fed: 0,
        }
    }

    pub(super) fn feed_word(&mut self, word: u64) {
        self.low = (self.low ^ word)
            .wrapping_mul(DIGEST_MULTIPLIER_LOW)
            .rotate_left(29);
        self.high = ((self.high ^ word.rotate_left(17)).wrapping_mul(DIGEST_MULTIPLIER_HIGH))
            .rotate_left(37)
            ^ self.low;
        self.words_fed += 1;
    }

    pub(super) fn feed_digest(&mut self, digest: Digest128) {
        self.feed_word(u64::try_from(digest.0 & u128::from(u64::MAX)).expect("低 64 位"));
        self.feed_word(u64::try_from(digest.0 >> 64).expect("高 64 位"));
    }

    /// 先喂长度，再按 8 字节小端逐字喂，尾巴补零成一个字。
    pub(super) fn feed_bytes(&mut self, bytes: &[u8]) {
        self.feed_word(u64::try_from(bytes.len()).expect("长度装得进 u64"));
        let (words, tail) = bytes.as_chunks::<8>();
        for word in words {
            self.feed_word(u64::from_le_bytes(*word));
        }
        if !tail.is_empty() {
            let mut padded = [0u8; 8];
            padded[..tail.len()].copy_from_slice(tail);
            self.feed_word(u64::from_le_bytes(padded));
        }
    }

    pub(super) fn finish(self) -> Digest128 {
        let low = avalanche_word(self.low ^ self.words_fed ^ self.high.rotate_left(13));
        let high = avalanche_word(self.high ^ low.rotate_left(7) ^ self.words_fed.rotate_left(40));
        Digest128((u128::from(high) << 64) | u128::from(low))
    }
}

/// 摘要的种类标签：内容、读集调用、键、结局指纹各自分开。
pub(super) const DIGEST_TAG_CONTENT: u64 = 0x636F_6E74_656E_7431;
pub(super) const DIGEST_TAG_CALL: u64 = 0x6361_6C6C_3132_3831;
pub(super) const DIGEST_TAG_KEY_SEQUENCE: u64 = 0x6B65_7973_6571_3131;
pub(super) const DIGEST_TAG_KEY_SET: u64 = 0x6B65_7973_6574_3131;
const DIGEST_TAG_OUTCOME: u64 = 0x6F75_7463_6F6D_6531;
pub(super) const DIGEST_TAG_PAIR: u64 = 0x7061_6972_3132_3831;
pub(super) const DIGEST_TAG_POSITION: u64 = 0x706F_7369_7469_6F6E;

pub(super) fn digest_of_bytes(bytes: &[u8]) -> Digest128 {
    let mut builder = DigestBuilder::new(DIGEST_TAG_CONTENT);
    builder.feed_bytes(bytes);
    builder.finish()
}

/// 结局的指纹：`Debug` 文本的摘要（`RecoveryReport` 与判定表都派生了 `Debug`，逐字段都在文本里；
/// 文本相同 ⇔ 值相同，除去 128 位摘要撞车，概率上界见登记 7.3）。
pub(super) fn fingerprint_of_debug_text(value: &dyn std::fmt::Debug) -> Digest128 {
    let mut builder = DigestBuilder::new(DIGEST_TAG_OUTCOME);
    builder.feed_bytes(format!("{value:?}").as_bytes());
    builder.finish()
}

#[cfg(test)]
mod tests;
