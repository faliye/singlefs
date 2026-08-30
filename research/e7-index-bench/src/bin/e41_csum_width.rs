//! E41：密文校验和位宽 —— D19（块指针的结构与宽度预算）已定项 2。
//!
//! ## 被测的那条推导（逐字）
//!
//! D19 已定项 2：「一条腿给出 32 位的推导：威胁模型是介质损坏不是攻击者，
//! 判据是随机损坏撞出相同校验值的概率 2⁻ⁿ，按 10 TB 池 / 每周 scrub / 10 年
//! ≈ 1.3×10¹² 次块读、损坏率 1e-9/块读估，要求漏检期望 < 1e-6 得 n > 30.3 位。
//! **⚠️ 这是单腿推导，未经校验路径复现，且损坏率那个数是假设不是实测。**」
//!
//! ## 两段
//!
//! 1. **算术段**：把漏检期望写成 `块读次数 × 损坏率 × 2⁻ⁿ` 的显式函数，
//!    对四个输入做敏感度扫描，给出**每个输入各自要错多少倍**才会把结论从 32 位推走。
//! 2. **碰撞段**：喂本工程会遇到的那几类损坏，实测漏检率，与 2⁻ⁿ 的理论值比。
//!
//! ⚠️ **32 位的漏检率在可行试验次数内测不出来**（要 ~4×10⁹ 次才等到一次漏检）。
//! 所以碰撞段测的是**截断到 8 / 12 / 16 位**的可测区间，先证「漏检率 = 2⁻ⁿ」这条律，
//! 再把 32 / 64 位作为**外推**报出去。外推这一步写明，不许当成实测。
//!
//! ## 阳性对照
//!
//! 8 位和校验是**故意弱**的那一臂。它有一个决定性的形态：
//! **字节交换它一次都检不出**（和不变），而 CRC 全检得出。
//! 测不出这个差别 ⇒ 损坏根本没注入进去，整轮作废。

use e7_index_bench::Emitter;

const BLOCK: usize = 512;
const TRIALS: u64 = 50_000;
const SEEDS: [u64; 5] = [1, 2, 3, 4, 5];
// SHA-256 那一臂慢，单独给一个较小的次数；分母各报各的，不许混用。
const SHA_TRIALS: u64 = 5_000;

// ── 确定性 PRNG：xorshift64*，纯整数，换机器不变 ──
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn fill(&mut self, buf: &mut [u8]) {
        for c in buf.chunks_mut(8) {
            let v = self.next().to_le_bytes();
            c.copy_from_slice(&v[..c.len()]);
        }
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

// ── CRC32C（Castagnoli，反射多项式 0x82F63B78）──
fn crc32c(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0x82F6_3B78 } else { crc >> 1 };
        }
    }
    !crc
}

// ── CRC64-ECMA（反射多项式 0xC96C5795D7870F42）──
fn crc64(data: &[u8]) -> u64 {
    let mut crc = !0u64;
    for &b in data {
        crc ^= b as u64;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xC96C_5795_D787_0F42 } else { crc >> 1 };
        }
    }
    !crc
}

// ── SHA-256（截断取低 32 位用）──
const K256: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256_low64(data: &[u8]) -> u64 {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut msg = data.to_vec();
    let bitlen = (data.len() as u64) * 8;
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bitlen.to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[4 * i], chunk[4 * i + 1], chunk[4 * i + 2], chunk[4 * i + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K256[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g; g = f; f = e; e = d.wrapping_add(t1);
            d = c; c = b; b = a; a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a); h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c); h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e); h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g); h[7] = h[7].wrapping_add(hh);
    }
    ((h[0] as u64) << 32) | h[1] as u64
}

/// 8 位和校验——**阳性对照那一臂，故意弱**。
fn sum8(data: &[u8]) -> u64 {
    data.iter().fold(0u8, |a, &b| a.wrapping_add(b)) as u64
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Alg {
    Sum8,
    Crc32cTrunc(u32),
    Crc32cFull,
    Crc64Full,
    Sha256Trunc32,
}

impl Alg {
    fn bits(self) -> u32 {
        match self {
            Alg::Sum8 => 8,
            Alg::Crc32cTrunc(n) => n,
            Alg::Crc32cFull => 32,
            Alg::Crc64Full => 64,
            Alg::Sha256Trunc32 => 32,
        }
    }
    fn tag(self) -> String {
        match self {
            Alg::Sum8 => "sum8".into(),
            Alg::Crc32cTrunc(n) => format!("crc32c_trunc{n}"),
            Alg::Crc32cFull => "crc32c_full32".into(),
            Alg::Crc64Full => "crc64_full64".into(),
            Alg::Sha256Trunc32 => "sha256_trunc32".into(),
        }
    }
    fn compute(self, data: &[u8]) -> u64 {
        match self {
            Alg::Sum8 => sum8(data),
            Alg::Crc32cTrunc(n) => (crc32c(data) as u64) & ((1u64 << n) - 1),
            Alg::Crc32cFull => crc32c(data) as u64,
            Alg::Crc64Full => crc64(data),
            Alg::Sha256Trunc32 => sha256_low64(data) & 0xFFFF_FFFF,
        }
    }
    fn trials(self) -> u64 {
        match self {
            Alg::Sha256Trunc32 => SHA_TRIALS,
            _ => TRIALS,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Damage {
    BitFlip,
    SectorZero,
    Misdirect,
    WholeBlock,
    ByteSwap,
}

impl Damage {
    fn tag(self) -> &'static str {
        match self {
            Damage::BitFlip => "bit_flip",
            Damage::SectorZero => "sector_zero",
            Damage::Misdirect => "misdirect",
            Damage::WholeBlock => "whole_block",
            Damage::ByteSwap => "byte_swap",
        }
    }
    /// 施加一次损坏。返回 false 表示这一次**没有真的改变字节**——
    /// 那种试验不计入分母，否则「漏检」里会混进「根本没坏」。
    fn apply(self, buf: &mut [u8], other: &[u8], rng: &mut Rng) -> bool {
        match self {
            Damage::BitFlip => {
                let i = rng.below(buf.len() as u64) as usize;
                let b = rng.below(8) as u32;
                buf[i] ^= 1 << b;
                true
            }
            Damage::SectorZero => {
                let was_zero = buf.iter().all(|&b| b == 0);
                buf.fill(0);
                !was_zero
            }
            Damage::Misdirect => {
                // 半个块被另一个块的对应半覆盖
                let half = buf.len() / 2;
                if buf[..half] == other[..half] {
                    return false;
                }
                buf[..half].copy_from_slice(&other[..half]);
                true
            }
            Damage::WholeBlock => {
                if buf == other {
                    return false;
                }
                buf.copy_from_slice(other);
                true
            }
            Damage::ByteSwap => {
                let i = rng.below(buf.len() as u64) as usize;
                let mut j = rng.below(buf.len() as u64) as usize;
                if i == j {
                    j = (j + 1) % buf.len();
                }
                if buf[i] == buf[j] {
                    return false; // 交换相同的两个字节等于什么都没做
                }
                buf.swap(i, j);
                true
            }
        }
    }
}

/// 一个 (算法, 损坏类, 种子) 的实测：注入了几次、漏检了几次。
/// **分母是实际注入次数**，由构造直接数出来，不从被测代码读回来。
fn measure(alg: Alg, dmg: Damage, seed: u64, trials: u64) -> (u64, u64) {
    let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let mut a = vec![0u8; BLOCK];
    let mut b = vec![0u8; BLOCK];
    let mut injected = 0u64;
    let mut missed = 0u64;
    for _ in 0..trials {
        rng.fill(&mut a);
        rng.fill(&mut b);
        let good = alg.compute(&a);
        let mut c = a.clone();
        if !dmg.apply(&mut c, &b, &mut rng) {
            continue; // 没真的改变字节，不计入分母
        }
        injected += 1;
        if alg.compute(&c) == good {
            missed += 1;
        }
    }
    (injected, missed)
}

// ── 算术段 ──

#[derive(Clone, Copy)]
struct Inputs {
    pool_bytes: f64,
    block_bytes: f64,
    scrub_days: f64,
    years: f64,
    corrupt_rate: f64,
    target: f64,
}

const BASE: Inputs = Inputs {
    pool_bytes: 10e12,   // 10 TB
    block_bytes: 4096.0, // 4 KiB
    scrub_days: 7.0,     // 每周
    years: 10.0,
    corrupt_rate: 1e-9, // 每块读
    target: 1e-6,       // 漏检期望上限
};

fn block_reads(i: Inputs) -> f64 {
    (i.pool_bytes / i.block_bytes) * (i.years * 365.0 / i.scrub_days)
}

fn corrupt_reads(i: Inputs) -> f64 {
    block_reads(i) * i.corrupt_rate
}

fn miss_expect(i: Inputs, n: u32) -> f64 {
    corrupt_reads(i) * 2f64.powi(-(n as i32))
}

/// 满足 `漏检期望 < target` 的最小位宽。
fn min_bits(i: Inputs) -> u32 {
    (1..=128).find(|&n| miss_expect(i, n) < i.target).unwrap_or(129)
}

/// 32 位那一档的余量：目标 ÷ 实际期望。**任何一个输入错这么多倍，结论就翻。**
fn margin_at(i: Inputs, n: u32) -> f64 {
    i.target / miss_expect(i, n)
}

fn main() {
    let mut em = Emitter::new();

    // ── 算术段：基线 ──
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=arith_base block_reads={:.6e} corrupt_reads={:.6e} min_bits={} \
             miss_at_32={:.6e} margin_at_32={:.4} miss_at_64={:.6e}",
            block_reads(BASE),
            corrupt_reads(BASE),
            min_bits(BASE),
            miss_expect(BASE, 32),
            margin_at(BASE, 32),
            miss_expect(BASE, 64),
        ))
    );

    // ── 算术段：每个输入各自要错多少倍才把 32 位推走 ──
    // 五个输入**全部线性**进入漏检期望（池容量、损坏率、服役年限成正比；
    // scrub 周期、块大小成反比）⇒ 每一个各自要错的倍数**都等于同一个余量**。
    // 逐条发出来是为了让检索到任一条的人都拿得到这个数，不必回头看别处。
    for (name, direction) in [
        ("pool_bytes", "proportional"),
        ("corrupt_rate", "proportional"),
        ("years", "proportional"),
        ("scrub_days", "inverse"),
        ("block_bytes", "inverse"),
    ] {
        // 校验路径：直接改这个输入，看最小位宽是不是真的从 31 跳到 32 以上
        let broken = match name {
            "pool_bytes" => Inputs { pool_bytes: BASE.pool_bytes * margin_at(BASE, 32) * 1.01, ..BASE },
            "corrupt_rate" => Inputs { corrupt_rate: BASE.corrupt_rate * margin_at(BASE, 32) * 1.01, ..BASE },
            "years" => Inputs { years: BASE.years * margin_at(BASE, 32) * 1.01, ..BASE },
            "scrub_days" => Inputs { scrub_days: BASE.scrub_days / (margin_at(BASE, 32) * 1.01), ..BASE },
            _ => Inputs { block_bytes: BASE.block_bytes / (margin_at(BASE, 32) * 1.01), ..BASE },
        };
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=sensitivity input={name} direction={direction} \
                 factor_to_break_32={:.4} min_bits_after_break={}",
                margin_at(BASE, 32),
                min_bits(broken),
            ))
        );
    }

    // ── 算术段：输入各错 10 / 100 / 1000 倍时的最小位宽 ──
    for f in [1.0f64, 10.0, 100.0, 1000.0, 1e4, 1e6] {
        let i = Inputs { corrupt_rate: BASE.corrupt_rate * f, ..BASE };
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=arith_scan corrupt_rate_factor={f:.0} corrupt_rate={:.3e} min_bits={} \
                 miss_at_32={:.6e}",
                i.corrupt_rate,
                min_bits(i),
                miss_expect(i, 32),
            ))
        );
    }

    // ── 碰撞段 ──
    let algs = [
        Alg::Sum8,
        Alg::Crc32cTrunc(8),
        Alg::Crc32cTrunc(12),
        Alg::Crc32cTrunc(16),
        Alg::Crc32cFull,
        Alg::Crc64Full,
        Alg::Sha256Trunc32,
    ];
    let dmgs = [
        Damage::BitFlip,
        Damage::SectorZero,
        Damage::Misdirect,
        Damage::WholeBlock,
        Damage::ByteSwap,
    ];
    for alg in algs {
        for dmg in dmgs {
            let mut inj = 0u64;
            let mut miss = 0u64;
            for s in SEEDS {
                let (i, m) = measure(alg, dmg, s, alg.trials());
                inj += i;
                miss += m;
            }
            // 理论值 2^-n，用「每百万次漏检数」表达，避免打印极小浮点
            let theory_ppm = 1e6 / 2f64.powi(alg.bits() as i32);
            let measured_ppm = if inj == 0 { 0.0 } else { 1e6 * miss as f64 / inj as f64 };
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=collide alg={} bits={} damage={} injected={inj} missed={miss} \
                     measured_ppm={measured_ppm:.3} theory_ppm={theory_ppm:.6}",
                    alg.tag(),
                    alg.bits(),
                    dmg.tag(),
                ))
            );
        }
    }

    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **CRC32C 的实现要对**：拿 Castagnoli 的公认测试向量钉死。
    /// `crc32c("123456789") = 0xE3069283`（RFC 3720 附录 B 的标准向量）。
    #[test]
    fn crc32c_matches_the_published_test_vector() {
        assert_eq!(crc32c(b"123456789"), 0xE306_9283);
        assert_eq!(crc32c(b""), 0);
    }

    /// **SHA-256 的实现要对**：空串的摘要前 8 字节是 `e3b0c442 98fc1c14`。
    #[test]
    fn sha256_matches_the_published_test_vector() {
        assert_eq!(sha256_low64(b""), 0xe3b0_c442_98fc_1c14);
        // "abc" 的摘要前 8 字节 ba7816bf 8f01cfea
        assert_eq!(sha256_low64(b"abc"), 0xba78_16bf_8f01_cfea);
    }

    /// **CRC64-ECMA 的实现要对**：`crc64("123456789") = 0x995DC9BBDF1939FA`。
    #[test]
    fn crc64_matches_the_published_test_vector() {
        assert_eq!(crc64(b"123456789"), 0x995D_C9BB_DF19_39FA);
    }

    /// **算术段要复现 D19 那条推导的三个数**：
    /// 10 TB / 4 KiB × (10 年 × 365 ÷ 7 天) ≈ 1.27×10¹² 次块读；
    /// × 1e-9 ⇒ 约 1273 次损坏读；要求 < 1e-6 ⇒ **最小位宽 31**。
    /// D19 原文写「n > 30.3」，取整就是 31 ⇒ 32 位够，但**只够一位**。
    #[test]
    fn the_arithmetic_reproduces_the_d19_derivation() {
        let br = block_reads(BASE);
        assert!(br > 1.2e12 && br < 1.4e12, "块读次数 {br:e} 不在 D19 说的 1.3e12 量级");
        assert_eq!(min_bits(BASE), 31);
        // 32 位的漏检期望
        let m = miss_expect(BASE, 32);
        assert!(m > 2.9e-7 && m < 3.0e-7, "32 位漏检期望 {m:e}");
    }

    /// **余量只有 3.4 倍**——这是本实验最要紧的那个数：
    /// 目标 1e-6 ÷ 32 位实际期望 2.96e-7 ≈ 3.37。
    /// ⇒ **三个输入里任何一个错 3.4 倍以上，32 位就不够了**，
    /// 而 D19 自陈损坏率那个数是假设不是实测。
    #[test]
    fn the_margin_at_32_bits_is_only_about_three_point_four() {
        let m = margin_at(BASE, 32);
        assert!(m > 3.3 && m < 3.5, "32 位的余量是 {m}");
        // 反过来：损坏率错 10 倍就要 34 位，错 1000 倍要 41 位
        assert_eq!(min_bits(Inputs { corrupt_rate: 1e-8, ..BASE }), 34);
        assert_eq!(min_bits(Inputs { corrupt_rate: 1e-6, ..BASE }), 41);
    }

    /// **阳性对照的决定性形态**：8 位和校验对**字节交换**一次都检不出（和不变）。
    /// 测不出这个差别 ⇒ 损坏根本没注入进去，整轮作废。
    #[test]
    fn positive_control_sum8_misses_every_byte_swap() {
        let (inj, miss) = measure(Alg::Sum8, Damage::ByteSwap, 1, 2000);
        assert!(inj > 1900, "注入次数 {inj} 太少，构造有问题");
        assert_eq!(miss, inj, "8 位和校验该对每一次字节交换都漏检");
    }

    /// **对照的另一半**：CRC32C 对同一批字节交换一次都不漏。
    /// 两条合起来才叫「测量有判别力」。
    #[test]
    fn crc32c_catches_every_byte_swap_the_sum_misses() {
        let (inj, miss) = measure(Alg::Crc32cFull, Damage::ByteSwap, 1, 2000);
        assert!(inj > 1900);
        assert_eq!(miss, 0, "CRC32C 不该漏检字节交换");
    }

    /// **截断到 8 位时漏检率必须落在 2⁻⁸ 附近**（二项分布 5σ 带）。
    /// 这条是把「漏检率 = 2⁻ⁿ」这条律钉死的地方——32 / 64 位靠它外推。
    #[test]
    fn truncated_crc_miss_rate_matches_two_to_the_minus_n() {
        let n = 8u32;
        let mut inj = 0u64;
        let mut miss = 0u64;
        for s in SEEDS {
            let (i, m) = measure(Alg::Crc32cTrunc(n), Damage::WholeBlock, s, 5000);
            inj += i;
            miss += m;
        }
        let p = 2f64.powi(-(n as i32));
        let mean = inj as f64 * p;
        let sigma = (inj as f64 * p * (1.0 - p)).sqrt();
        assert!(inj > 24_000, "注入 {inj} 次太少");
        assert!(
            (miss as f64 - mean).abs() < 5.0 * sigma,
            "8 位截断漏检 {miss} 次，期望 {mean:.1} ± {sigma:.1}"
        );
    }

    /// **分母就是实际注入次数**，不从被测代码读回来：
    /// 没真的改变字节的那些试验不计入。
    #[test]
    fn the_denominator_is_the_injected_count_by_construction() {
        let (inj, miss) = measure(Alg::Crc32cFull, Damage::BitFlip, 7, 1000);
        assert_eq!(inj, 1000, "单比特翻转每次都真的改了字节");
        assert_eq!(miss, 0, "CRC32C 检得出全部单比特翻转");
        // 而「整块替换成另一个随机块」偶尔会撞上相同的块（本实验里不会，但分母要照数）
        let (inj2, _) = measure(Alg::Crc32cFull, Damage::WholeBlock, 7, 1000);
        assert_eq!(inj2, 1000);
    }

    /// **CRC 的结构性质**：单比特翻转的漏检率**恒为 0**，不是 2⁻ⁿ。
    /// ⇒ 「漏检率 = 2⁻ⁿ」只对「整块替换」这类**随机化**损坏成立，
    /// 拿它当全部损坏类的通用式子会低估 CRC。
    #[test]
    fn crc_beats_two_to_the_minus_n_on_single_bit_flips() {
        for s in SEEDS {
            let (_, m32) = measure(Alg::Crc32cFull, Damage::BitFlip, s, 2000);
            let (_, m64) = measure(Alg::Crc64Full, Damage::BitFlip, s, 2000);
            assert_eq!(m32, 0);
            assert_eq!(m64, 0);
        }
    }
}
