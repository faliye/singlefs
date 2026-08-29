//! E6 的**多核扩展档**：AES-256-GCM 与 ChaCha20-Poly1305 的吞吐随线程数怎么走。
//!
//! **为什么必须补这一档**：[decisions.md] D24 判「有 AES-NI ⇒ GPU 卸载是负收益」，
//! 靠的是「单核 3888 MiB/s × 16 核 ≈ 65 GB/s > GPU 端到端 50.3 GB/s」。
//! 而 [experiments.md] E6 口径节逐字写着「**不是多核扩展比。** …本轮未测」，
//! ⇒ **那一步线性外推没有依据**，而 E21 实测 XOR 折叠两线程就撞上内存带宽墙。
//! 本实验就测那一步。
//!
//! ## 两条阳性对照，缺一不可
//!
//! 1. **单核吞吐必须复现 E6**（4 KiB 档 3888.51 MiB/s，容差 ±20%）。
//!    对不上说明本 harness 与 E6 测的不是同一件事，**整轮作废**——
//!    那时多核那一列再漂亮也不能拿去和 E6 的单核数并列。
//!    ⚠️ E6 在 QEMU/KVM 来宾里跑，本实验在宿主上跑，所以容差放宽到 ±20%。
//! 2. **算力受限臂必须接近线性扩展**（固定总工作量，16 线程 ≥ 4×）。
//!    测不出加速说明 harness 看不见并行度，**整轮作废**——
//!    不许把「加密不扩展」这个结论建立在一个看不见并行的 harness 上。
//!
//! ## 缓冲大小是自变量，不是常数
//!
//! 原地加密要读一遍写一遍 ⇒ 内存流量是吞吐的两倍。
//! 每线程的工作集若装进 L2/L3，量到的是缓存带宽；装不进才是内存带宽。
//! **全盘 scrub / 整卷加密属于后者**，所以两档都测，并报清楚。

// ⚠️ **故意用已弃用的 `encrypt_in_place_detached`，不换成 `encrypt_inout_detached`。**
// 阳性对照 1 要求单核吞吐复现 E6，而 E6 口径写明用的就是 `encrypt_in_place_detached`。
// 换 API 就换了被测对象，那个对照当场失去意义。
// ⇒ 这里压制弃用警告是**有理由的压制**，不是忽略警告
// （`.claude/singlefs-ai-sop/rules/command-safety.md`：警告是免费的信号，不许略过）。
#![allow(deprecated)]

use aes_gcm::{aead::{AeadInPlace, KeyInit}, Aes256Gcm, Nonce};
use chacha20poly1305::ChaCha20Poly1305;
use e7_index_bench::Emitter;
use std::time::Instant;

const UNIT: usize = 4096;
/// E6 4 KiB 档的单核实测值（MiB/s），用作阳性对照 1 的基准。
const E6_AES_4K_MIBS: f64 = 3888.51;

#[derive(Clone, Copy, PartialEq)]
enum Alg { Aes, Chacha }
impl Alg {
    fn name(self) -> &'static str {
        match self {
            // 没有 `_ =>` —— 新增算法不补这里就编译不过
            Alg::Aes => "aes256gcm",
            Alg::Chacha => "chacha20poly1305",
        }
    }
}

/// 原地加密 `buf` 一整遍，按 `UNIT` 分块。返回处理的字节数。
/// **缓冲预分配、循环内不分配**——否则测到的是 allocator 不是算法（E6 口径原话）。
fn seal(alg: Alg, buf: &mut [u8]) -> usize {
    let key = [7u8; 32];
    let nonce = Nonce::from_slice(&[0u8; 12]);
    let mut tag_sink = [0u8; 16];
    let n = buf.len();
    match alg {
        Alg::Aes => {
            let c = Aes256Gcm::new_from_slice(&key).unwrap();
            for ch in buf.chunks_mut(UNIT) {
                let t = c.encrypt_in_place_detached(nonce, b"", ch).unwrap();
                tag_sink.copy_from_slice(&t);
            }
        }
        Alg::Chacha => {
            let c = ChaCha20Poly1305::new_from_slice(&key).unwrap();
            for ch in buf.chunks_mut(UNIT) {
                let t = c.encrypt_in_place_detached(nonce, b"", ch).unwrap();
                tag_sink.copy_from_slice(&t);
            }
        }
    }
    std::hint::black_box(&tag_sink);
    n
}

/// 跑一档：`threads` 条线程，每条一份 `per_thread_mib` 的独立缓冲。返回 (纳秒, 总字节)。
fn run(alg: Alg, threads: usize, per_thread_mib: usize) -> (u64, usize) {
    let bytes = per_thread_mib * 1024 * 1024;
    // 缓冲在计时之外分配并预热
    let mut bufs: Vec<Vec<u8>> = (0..threads).map(|_| vec![0u8; bytes]).collect();
    // ⚠️ **必须逐页预热。** `vec![0u8; n]` 给的是惰性映射的零页，只碰首尾两个字节的话
    // **计时区里会在缺页**——实测那样单核只有 2154 MiB/s，与 E6 的 3888 差 45%，
    // 阳性对照当场判红。这是 harness 的错，不是算法慢。
    for b in bufs.iter_mut() {
        for p in b.chunks_mut(4096) { p[0] = 1; }
    }
    let t0 = Instant::now();
    let mut hs = Vec::with_capacity(threads);
    for mut b in bufs.into_iter() {
        hs.push(std::thread::spawn(move || { let n = seal(alg, &mut b); std::hint::black_box(&b); n }));
    }
    let total: usize = hs.into_iter().map(|h| h.join().unwrap()).sum();
    (t0.elapsed().as_nanos() as u64, total)
}

/// **阳性对照 2 的臂：算力受限。** 固定总工作量按线程均分 ⇒ 完美并行时比值 ≈ 线程数。
/// ⚠️ 让每条线程都做同样多次是弱扩展，那时完美并行的比值是 1，会把结论判反（E21 踩过）。
fn compute(threads: usize, total: u64) -> u64 {
    let iters = total / threads as u64;
    let t0 = Instant::now();
    let mut hs = Vec::with_capacity(threads);
    for t in 0..threads {
        hs.push(std::thread::spawn(move || {
            let mut x = t as u64 | 1;
            for _ in 0..iters { x = x.wrapping_mul(0x9E37_79B9_7F4A_7C15).rotate_left(23) ^ 0x5851_F42D_4C95_7F2D; }
            x
        }));
    }
    let mut acc = 0u64;
    for h in hs { acc ^= h.join().unwrap(); }
    std::hint::black_box(acc);
    t0.elapsed().as_nanos() as u64
}

fn mibs(bytes: usize, ns: u64) -> f64 {
    if ns == 0 { return f64::NAN; }
    bytes as f64 / (1024.0 * 1024.0) / (ns as f64 / 1e9)
}

fn main() {
    let rounds: usize = std::env::args().nth(1).and_then(|x| x.parse().ok()).unwrap_or(3);
    let cores = std::thread::available_parallelism().map(|v| v.get()).unwrap_or(1);
    let mut em = Emitter::new();
    let mut out = String::new();
    let mut say = |s: String| { out.push_str(&s); out.push('\n'); };
    let die = |out: &mut String, em: &mut Emitter, msg: &str| -> ! {
        out.push_str(&em.finish()); out.push('\n'); print!("{out}");
        eprintln!("E6MC: {msg}"); std::process::exit(4);
    };

    say(em.emit_raw(&format!("name=config unit={UNIT} rounds={rounds} cores={cores} e6_aes_4k_mibs={E6_AES_4K_MIBS}")));

    // ── 阳性对照 1：单核 AES 必须复现 E6 ──
    let mut best = u64::MAX; let mut nb = 0;
    for _ in 0..rounds { let (ns, b) = run(Alg::Aes, 1, 64); if ns < best { best = ns; nb = b; } }
    let single = mibs(nb, best);
    let dev = (single - E6_AES_4K_MIBS).abs() / E6_AES_4K_MIBS;
    say(em.emit_raw(&format!("name=poscontrol arm=e6_single_core mibs={single:.2} e6={E6_AES_4K_MIBS} dev={:.1}% ok={}", dev*100.0, dev <= 0.20)));
    if dev > 0.20 {
        die(&mut out, &mut em, &format!("单核 {single:.0} MiB/s 与 E6 的 {E6_AES_4K_MIBS} 差 {:.0}%（>20%）—— 本 harness 与 E6 测的不是同一件事，整轮作废", dev*100.0));
    }

    // ── 阳性对照 2：算力受限臂必须接近线性扩展 ──
    let c1 = (0..3).map(|_| compute(1, 800_000_000)).min().unwrap();
    let c16 = (0..3).map(|_| compute(16, 800_000_000)).min().unwrap();
    let sp = c1 as f64 / c16 as f64;
    say(em.emit_raw(&format!("name=poscontrol arm=compute threads=16 speedup={sp:.2} ok={}", sp >= 4.0)));
    if sp < 4.0 { die(&mut out, &mut em, &format!("算力受限臂 16 线程只加速 {sp:.2}×（要求 ≥4）—— harness 看不见并行度，整轮作废")); }

    // ── 主体：两种算法 × 两档工作集 × 线程数扫描 ──
    for (tag, per_thread_mib) in [("l2_resident", 1usize), ("dram", 64usize)] {
        for alg in [Alg::Aes, Alg::Chacha] {
            let mut base = 0.0f64;
            for &th in &[1usize, 2, 4, 8, 16, 32] {
                if th > cores { continue; }
                let mut b = u64::MAX; let mut tb = 0;
                for _ in 0..rounds { let (ns, by) = run(alg, th, per_thread_mib); if ns < b { b = ns; tb = by; } }
                let m = mibs(tb, b);
                if th == 1 { base = m; }
                say(em.emit_raw(&format!(
                    "name=scale workset={tag} per_thread_mib={per_thread_mib} alg={} threads={th} mibs={m:.1} gbps={:.2} speedup={:.2}",
                    alg.name(), m * 1024.0 * 1024.0 / 1e9, m / base)));
            }
        }
    }
    say(em.finish());
    print!("{out}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 加密必须真的改变缓冲内容——否则量到的是一个空循环。
    #[test]
    fn seal_actually_transforms_the_buffer() {
        for alg in [Alg::Aes, Alg::Chacha] {
            let mut b = vec![0u8; UNIT * 4];
            let before = b.clone();
            let n = seal(alg, &mut b);
            assert_eq!(n, UNIT * 4);
            assert_ne!(b, before, "{} 没有改变缓冲，测的是空循环", alg.name());
        }
    }

    /// 两种算法必须产出不同的密文——否则枚举分派串了。
    #[test]
    fn the_two_algorithms_differ() {
        let (mut a, mut c) = (vec![0u8; UNIT], vec![0u8; UNIT]);
        seal(Alg::Aes, &mut a); seal(Alg::Chacha, &mut c);
        assert_ne!(a, c, "两种算法产出相同密文，分派串了");
    }

    /// 每个 UNIT 都要被加密到，不是只加密第一块。
    #[test]
    fn every_unit_is_covered() {
        let mut b = vec![0u8; UNIT * 3];
        seal(Alg::Aes, &mut b);
        for (i, ch) in b.chunks(UNIT).enumerate() {
            assert!(ch.iter().any(|&x| x != 0), "第 {i} 个单元没被加密");
        }
    }

    /// 算力受限对照必须固定总工作量：线程数翻倍，单线程迭代数减半。
    #[test]
    fn compute_control_holds_total_work_constant() {
        // 用极小的总量，只验分派逻辑不验性能
        let a = compute(1, 1_000_000);
        let b = compute(4, 1_000_000);
        assert!(a > 0 && b > 0);
        assert!(b < a, "4 线程做同样总量应当更快，实测 {b} vs {a}");
    }
}
