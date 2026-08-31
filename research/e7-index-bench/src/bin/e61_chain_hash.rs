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
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0x82F6_3B78 } else { crc >> 1 };
        }
    }
    !crc
}

/// 丙（阳性对照）：CRC-64/XZ，取低 32 位。反射多项式 0xC96C5795D7870F42。
fn crc64_xz(data: &[u8]) -> u64 {
    let mut crc: u64 = 0xFFFF_FFFF_FFFF_FFFF;
    for &b in data {
        crc ^= b as u64;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xC96C_5795_D787_0F42 } else { crc >> 1 };
        }
    }
    !crc
}
fn crc64_trunc32(data: &[u8]) -> u32 {
    crc64_xz(data) as u32
}

/// 乙：SHA-256，取前 4 字节（大端）。从头实现，由 FIPS 180-4 的测试向量钉死。
fn sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
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
        let (mut a, mut b, mut c, mut d) = (h[0], h[1], h[2], h[3]);
        let (mut e, mut f, mut g, mut hh) = (h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g; g = f; f = e; e = d.wrapping_add(t1);
            d = c; c = b; b = a; a = t1.wrapping_add(t2);
        }
        for (i, v) in [a, b, c, d, e, f, g, hh].into_iter().enumerate() {
            h[i] = h[i].wrapping_add(v);
        }
    }
    let mut out = [0u8; 32];
    for i in 0..8 {
        out[4 * i..4 * i + 4].copy_from_slice(&h[i].to_be_bytes());
    }
    out
}
fn sha256_trunc32(data: &[u8]) -> u32 {
    let d = sha256(data);
    u32::from_be_bytes([d[0], d[1], d[2], d[3]])
}

// ───────────────────────── 记录头模型（E23（journal 几何） 的 11 个字段）─────────────────────────

/// `header_csum` 那 32 字节不进 hash 输入（D23（journal 的角色与格式） 已定项 10 的实现纪律：当零）。
const CSUM_BYTES: usize = 32;
/// ⚠️ 这三档是**跑这一轮时**按当时的字段表建的模（86 = 含 `tail_lsn` 那 8 字节的版本）。
/// 2026-08-31 `tail_lsn` 随 D23（journal 的角色与格式） 已定项 3 去掉 ⇒ 现行是 78 / 87 / 91。
/// 本文件连同它的产物是那一轮的记录，**不跟着改**——改了产物就不再对应它的输入。
/// E61（反向链 hash 算法的均匀性） 的量二结论已作废，见该实验正文开头。
const HDR_LENS: [usize; 3] = [86, 95, 99];

/// 造一个形态真实的记录头：可变字段填满，`header_csum` 段清零。
fn make_header(hdr_len: usize, jsn: u64, epoch: u32, txg: u64) -> Vec<u8> {
    let mut h = vec![0u8; hdr_len];
    h[0..4].copy_from_slice(&0x6A_53_4E_31u32.to_be_bytes()); // magic
    h[4..6].copy_from_slice(&1u16.to_be_bytes());             // version/type
    h[6] = 0;                                                 // algo_type
    h[7] = 0;                                                 // 对齐填充
    h[8..12].copy_from_slice(&(hdr_len as u32).to_be_bytes()); // record_length
    h[12..16].copy_from_slice(&7u32.to_be_bytes());            // named_count
    h[16..20].copy_from_slice(&epoch.to_be_bytes());           // jsn 高段：实例代号 32 位
    h[20..26].copy_from_slice(&jsn.to_be_bytes()[2..]);        // jsn 低段：计数器 48 位
    h[26..34].copy_from_slice(&txg.to_be_bytes());             // checkpoint_txg
    h[34..42].copy_from_slice(&(jsn ^ 0x5A5A).to_be_bytes());  // tail_lsn
    for i in 42..54 {
        h[i] = (jsn as u8).wrapping_add(i as u8);              // nonce 12 字节
    }
    // 54..54+32 是 header_csum，按已定项 10 的纪律留零，不进输入
    h
}
/// 可变区（差异只可能落在这里）的比特数。
fn variable_bits(hdr_len: usize) -> usize {
    (hdr_len - CSUM_BYTES) * 8
}

// ───────────────────────── GF(2) 秩 ─────────────────────────

/// 32 个像向量在 GF(2) 上的秩。秩 = 32 ⟺ 该窗口内 2³² 个差异一个都不碰撞。
fn gf2_rank(mut v: Vec<u32>) -> usize {
    let mut rank = 0;
    for bit in (0..32).rev() {
        if let Some(p) = (rank..v.len()).find(|&i| v[i] >> bit & 1 == 1) {
            v.swap(rank, p);
            let pivot = v[rank];
            for i in 0..v.len() {
                if i != rank && (v[i] >> bit & 1 == 1) {
                    v[i] ^= pivot;
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
fn colliding_diffs(rank: usize) -> u64 {
    (1u64 << (32 - rank as u32)) - 1
}

/// 一个 32 比特窗口的像基：`f(base XOR e_i) XOR f(base)`。
/// 对线性的 f（CRC 及其截断）这与 base 无关——`base_independent` 那个单测钉的就是这一条。
fn window_image<F: Fn(&[u8]) -> u32>(f: &F, base: &[u8], bit_off: usize) -> Vec<u32> {
    let h0 = f(base);
    (0..32)
        .map(|k| {
            let mut m = base.to_vec();
            let b = bit_off + k;
            m[b / 8] ^= 1u8 << (b % 8);
            f(&m) ^ h0
        })
        .collect()
}

/// 扫遍可变区里所有 32 比特窗口，报「秩满的窗口数 / 总窗口数」与最差秩。
fn rank_spectrum<F: Fn(&[u8]) -> u32>(f: &F, hdr_len: usize) -> (usize, usize, usize, u64) {
    let base = make_header(hdr_len, 0x0000_1234_5678, 0xDEAD_BEEF, 0x00AA);
    let vb = variable_bits(hdr_len);
    let (mut full, mut total, mut worst) = (0usize, 0usize, 32usize);
    let mut worst_collisions: u64 = 0;
    for off in 0..=(vb - 32) {
        let r = gf2_rank(window_image(f, &base, off));
        total += 1;
        if r == 32 { full += 1; } else if r < worst {
            worst = r;
            worst_collisions = colliding_diffs(r);
        }
    }
    (full, total, worst, worst_collisions)
}

// ───────────────────────── 抽样（非线性臂只能这么问）─────────────────────────

fn splitmix64(s: &mut u64) -> u64 {
    *s = s.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *s;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// 生日法：N 个互不相同的记录头，数碰撞。期望 = N(N−1)/2 / 2³²。
fn birthday<F: Fn(&[u8]) -> u32>(f: &F, hdr_len: usize, n: usize, seed: u64) -> (u64, f64) {
    let mut s = seed;
    let mut seen: HashMap<u32, u32> = HashMap::with_capacity(n * 2);
    let mut hits: u64 = 0;
    for _ in 0..n {
        let h = make_header(
            hdr_len,
            splitmix64(&mut s) & 0xFFFF_FFFF_FFFF,
            splitmix64(&mut s) as u32,
            splitmix64(&mut s),
        );
        let v = f(&h);
        let e = seen.entry(v).or_insert(0);
        hits += *e as u64;
        *e += 1;
    }
    let expect = (n as f64) * ((n - 1) as f64) / 2.0 / 4_294_967_296.0;
    (hits, expect)
}

// ───────────────────────── 输出 ─────────────────────────

struct Emitter(u32);
impl Emitter {
    fn emit(&mut self, s: String) {
        self.0 += 1;
        println!("E7RESULT {s}");
    }
}

fn main() {
    let mut em = Emitter(0);
    em.emit(format!(
        "name=config csum_bytes={CSUM_BYTES} hdr_lens=86/95/99 birthday_n=1048576"
    ));

    // ── 量一：秩谱。甲与丙线性，可精确判；乙不线性，不适用 ──
    for &hl in &HDR_LENS {
        for (arm, name) in [
            (&crc32c as &dyn Fn(&[u8]) -> u32, "jia_crc32c"),
            (&crc64_trunc32 as &dyn Fn(&[u8]) -> u32, "bing_crc64_trunc32"),
        ] {
            let f = |d: &[u8]| arm(d);
            let (full, total, worst, coll) = rank_spectrum(&f, hl);
            em.emit(format!(
                "name=rank_spectrum arm={name} hdr_len={hl} windows_full_rank={full} windows_total={total} \
                 worst_rank={worst} worst_window_collisions={coll}"
            ));
        }
    }

    // ── 量二：结构性差异——只有 jsn 的实例代号那 4 字节不同（两条时间线的实际形态）──
    for &hl in &HDR_LENS {
        for (arm, name) in [
            (&crc32c as &dyn Fn(&[u8]) -> u32, "jia_crc32c"),
            (&crc64_trunc32 as &dyn Fn(&[u8]) -> u32, "bing_crc64_trunc32"),
        ] {
            let f = |d: &[u8]| arm(d);
            let base = make_header(hl, 0x0000_1234_5678, 0xDEAD_BEEF, 0x00AA);
            let r = gf2_rank(window_image(&f, &base, 16 * 8)); // 实例代号在字节 16..20
            em.emit(format!(
                "name=epoch_window arm={name} hdr_len={hl} rank={r} collisions={}",
                colliding_diffs(r)
            ));
        }
    }

    // ── 量三：抽样。三条臂都跑，乙是唯一只能靠它的 ──
    for &hl in &HDR_LENS {
        for (arm, name) in [
            (&crc32c as &dyn Fn(&[u8]) -> u32, "jia_crc32c"),
            (&sha256_trunc32 as &dyn Fn(&[u8]) -> u32, "yi_sha256_trunc32"),
            (&crc64_trunc32 as &dyn Fn(&[u8]) -> u32, "bing_crc64_trunc32"),
        ] {
            let f = |d: &[u8]| arm(d);
            let (hits, expect) = birthday(&f, hl, 1 << 20, 0x1234_5678_9ABC_DEF0);
            em.emit(format!(
                "name=birthday arm={name} hdr_len={hl} collisions={hits} expect={expect:.2} ratio={:.3}",
                hits as f64 / expect
            ));
        }
    }

    // ── 量四：同源必须恒等（E51（反向链的碰撞机会有多少次） 判据 1 同型）──
    let h = make_header(86, 42, 7, 9);
    em.emit(format!(
        "name=same_source crc32c_equal={} sha_equal={} crc64_equal={}",
        crc32c(&h) == crc32c(&h),
        sha256_trunc32(&h) == sha256_trunc32(&h),
        crc64_trunc32(&h) == crc64_trunc32(&h)
    ));

    println!("E7RESULT name=done emitted={}", em.0 + 1);
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
    fn crc64_matches_the_published_vector() {
        assert_eq!(crc64_xz(b"123456789"), 0x995D_C9BB_DF19_39FA);
        assert_eq!(crc64_trunc32(b"123456789"), 0xDF19_39FA);
    }

    /// 判据 3 之三：SHA-256 的 FIPS 180-4 测试向量。
    #[test]
    fn sha256_matches_the_published_vector() {
        let d = sha256(b"abc");
        assert_eq!(&d[..4], &[0xBA, 0x78, 0x16, 0xBF]);
        assert_eq!(sha256_trunc32(b"abc"), 0xBA78_16BF);
        // 空串向量，覆盖 padding 的另一条路径
        assert_eq!(sha256_trunc32(b""), 0xE3B0_C442);
    }

    /// 秩判据成立的前提：CRC 的差分与 base 无关（线性）。不成立则整个量一无意义。
    #[test]
    fn crc_difference_is_base_independent() {
        let a = make_header(86, 1, 1, 1);
        let b = make_header(86, 999, 12345, 777);
        for off in [0usize, 16 * 8, 40 * 8] {
            assert_eq!(
                window_image(&crc32c, &a, off),
                window_image(&crc32c, &b, off),
                "CRC32C 的差分依赖了 base，说明它不是线性映射"
            );
        }
    }

    /// **阳性对照本身要先证明它测得出差别**：SHA-256 的差分必须依赖 base，
    /// 否则「乙不能用秩判据」这个结论是空的。
    #[test]
    fn sha_difference_is_not_base_independent() {
        let a = make_header(86, 1, 1, 1);
        let b = make_header(86, 999, 12345, 777);
        assert_ne!(window_image(&sha256_trunc32, &a, 0), window_image(&sha256_trunc32, &b, 0));
    }

    /// 报数口径的绝对值断言：满秩 ⇒ 0 个碰撞，不是 1 个（1 是核里那个零向量，属同源比较）。
    #[test]
    fn colliding_diffs_excludes_the_zero_vector() {
        assert_eq!(colliding_diffs(32), 0);
        assert_eq!(colliding_diffs(31), 1);
        assert_eq!(colliding_diffs(28), 15);
    }

    /// GF(2) 求秩本身的判别力：满秩、缺一秩、零矩阵三档都要判对。
    #[test]
    fn gf2_rank_has_discrimination() {
        let full: Vec<u32> = (0..32).map(|i| 1u32 << i).collect();
        assert_eq!(gf2_rank(full), 32);
        let mut deficient: Vec<u32> = (0..32).map(|i| 1u32 << i).collect();
        deficient[31] = deficient[0] ^ deficient[1]; // 制造一条线性相关
        assert_eq!(gf2_rank(deficient), 31);
        assert_eq!(gf2_rank(vec![0u32; 32]), 0);
    }

    /// 记录头模型的绝对值断言：三档长度、可变区比特数、csum 段必须为零。
    #[test]
    fn header_model_is_pinned() {
        assert_eq!(HDR_LENS, [86, 95, 99]);
        assert_eq!(variable_bits(86), 54 * 8);
        assert_eq!(variable_bits(99), 67 * 8);
        let h = make_header(86, 42, 7, 9);
        assert_eq!(h.len(), 86);
        assert!(h[54..86].iter().all(|&b| b == 0), "header_csum 段必须留零，它不进 hash 输入");
    }

    /// 同源比较不构成碰撞机会：同一份输入必须构造性相等。
    #[test]
    fn same_source_is_constructively_equal() {
        let h = make_header(95, 12345, 999, 42);
        assert_eq!(crc32c(&h), crc32c(&h));
        assert_eq!(sha256_trunc32(&h), sha256_trunc32(&h));
    }
}
