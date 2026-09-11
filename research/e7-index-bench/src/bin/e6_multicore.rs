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

const ENCRYPTION_UNIT_BYTES: usize = 4096;
/// E6 4 KiB 档的单核实测值（MiB/s），用作阳性对照 1 的基准。
const E6_AES_4_KIBIBYTE_TIER_MEBIBYTES_PER_SECOND: f64 = 3888.51;

#[derive(Clone, Copy, PartialEq)]
enum CipherAlgorithm { Aes, Chacha }
impl CipherAlgorithm {
    fn name(self) -> &'static str {
        match self {
            // 没有 `_ =>` —— 新增算法不补这里就编译不过
            CipherAlgorithm::Aes => "aes256gcm",
            CipherAlgorithm::Chacha => "chacha20poly1305",
        }
    }
}

/// 原地加密 `buffer` 一整遍，按 `ENCRYPTION_UNIT_BYTES` 分块。返回处理的字节数。
/// **缓冲预分配、循环内不分配**——否则测到的是 allocator 不是算法（E6 口径原话）。
fn seal_buffer_in_place(cipher_algorithm: CipherAlgorithm, buffer: &mut [u8]) -> usize {
    let key = [7u8; 32];
    let nonce = Nonce::from_slice(&[0u8; 12]);
    let mut tag_sink = [0u8; 16];
    let buffer_byte_count = buffer.len();
    match cipher_algorithm {
        CipherAlgorithm::Aes => {
            let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
            for unit_chunk in buffer.chunks_mut(ENCRYPTION_UNIT_BYTES) {
                let authentication_tag = cipher.encrypt_in_place_detached(nonce, b"", unit_chunk).unwrap();
                tag_sink.copy_from_slice(&authentication_tag);
            }
        }
        CipherAlgorithm::Chacha => {
            let cipher = ChaCha20Poly1305::new_from_slice(&key).unwrap();
            for unit_chunk in buffer.chunks_mut(ENCRYPTION_UNIT_BYTES) {
                let authentication_tag = cipher.encrypt_in_place_detached(nonce, b"", unit_chunk).unwrap();
                tag_sink.copy_from_slice(&authentication_tag);
            }
        }
    }
    std::hint::black_box(&tag_sink);
    buffer_byte_count
}

/// 跑一档：`thread_count` 条线程，每条一份 `per_thread_mebibytes` 的独立缓冲。返回 (纳秒, 总字节)。
fn time_parallel_seal_in_nanoseconds(cipher_algorithm: CipherAlgorithm, thread_count: usize, per_thread_mebibytes: usize) -> (u64, usize) {
    let per_thread_byte_count = per_thread_mebibytes * 1024 * 1024;
    // 缓冲在计时之外分配并预热
    let mut thread_buffers: Vec<Vec<u8>> = (0..thread_count).map(|_| vec![0u8; per_thread_byte_count]).collect();
    // ⚠️ **必须逐页预热。** `vec![0u8; n]` 给的是惰性映射的零页，只碰首尾两个字节的话
    // **计时区里会在缺页**——实测那样单核只有 2154 MiB/s，与 E6 的 3888 差 45%，
    // 阳性对照当场判红。这是 harness 的错，不是算法慢。
    for thread_buffer in thread_buffers.iter_mut() {
        for page in thread_buffer.chunks_mut(4096) { page[0] = 1; }
    }
    let start_instant = Instant::now();
    let mut worker_handles = Vec::with_capacity(thread_count);
    for mut thread_buffer in thread_buffers.into_iter() {
        worker_handles.push(std::thread::spawn(move || { let sealed_byte_count = seal_buffer_in_place(cipher_algorithm, &mut thread_buffer); std::hint::black_box(&thread_buffer); sealed_byte_count }));
    }
    let total_sealed_byte_count: usize = worker_handles.into_iter().map(|worker_handle| worker_handle.join().unwrap()).sum();
    (start_instant.elapsed().as_nanos() as u64, total_sealed_byte_count)
}

/// **阳性对照 2 的臂：算力受限。** 固定总工作量按线程均分 ⇒ 完美并行时比值 ≈ 线程数。
/// ⚠️ 让每条线程都做同样多次是弱扩展，那时完美并行的比值是 1，会把结论判反（E21 踩过）。
fn compute(thread_count: usize, total_iterations: u64) -> u64 {
    let iterations_per_thread = total_iterations / thread_count as u64;
    let start_instant = Instant::now();
    let mut worker_handles = Vec::with_capacity(thread_count);
    for thread_index in 0..thread_count {
        worker_handles.push(std::thread::spawn(move || {
            let mut chain_state = thread_index as u64 | 1;
            for _ in 0..iterations_per_thread { chain_state = chain_state.wrapping_mul(0x9E37_79B9_7F4A_7C15).rotate_left(23) ^ 0x5851_F42D_4C95_7F2D; }
            chain_state
        }));
    }
    let mut combined_chain_state = 0u64;
    for worker_handle in worker_handles { combined_chain_state ^= worker_handle.join().unwrap(); }
    std::hint::black_box(combined_chain_state);
    start_instant.elapsed().as_nanos() as u64
}

fn throughput_in_mebibytes_per_second(byte_count: usize, elapsed_nanoseconds: u64) -> f64 {
    if elapsed_nanoseconds == 0 { return f64::NAN; }
    byte_count as f64 / (1024.0 * 1024.0) / (elapsed_nanoseconds as f64 / 1e9)
}

fn main() {
    let round_count: usize = std::env::args().nth(1).and_then(|argument| argument.parse().ok()).unwrap_or(3);
    let core_count = std::thread::available_parallelism().map(|parallelism| parallelism.get()).unwrap_or(1);
    let mut emitter = Emitter::new();
    let mut output_text = String::new();
    let mut append_output_line = |output_line: String| { output_text.push_str(&output_line); output_text.push('\n'); };
    let flush_output_and_abort_round = |output_text: &mut String, emitter: &mut Emitter, failure_message: &str| -> ! {
        output_text.push_str(&emitter.finish()); output_text.push('\n'); print!("{output_text}");
        eprintln!("E6MC: {failure_message}"); std::process::exit(4);
    };

    append_output_line(emitter.emit_raw(&format!("name=config unit={ENCRYPTION_UNIT_BYTES} rounds={round_count} cores={core_count} e6_aes_4k_mibs={E6_AES_4_KIBIBYTE_TIER_MEBIBYTES_PER_SECOND}")));

    // ── 阳性对照 1：单核 AES 必须复现 E6 ──
    let mut single_core_best_nanoseconds = u64::MAX; let mut single_core_best_byte_count = 0;
    for _ in 0..round_count { let (elapsed_nanoseconds, sealed_byte_count) = time_parallel_seal_in_nanoseconds(CipherAlgorithm::Aes, 1, 64); if elapsed_nanoseconds < single_core_best_nanoseconds { single_core_best_nanoseconds = elapsed_nanoseconds; single_core_best_byte_count = sealed_byte_count; } }
    let single_core_mebibytes_per_second = throughput_in_mebibytes_per_second(single_core_best_byte_count, single_core_best_nanoseconds);
    let deviation_from_e6 = (single_core_mebibytes_per_second - E6_AES_4_KIBIBYTE_TIER_MEBIBYTES_PER_SECOND).abs() / E6_AES_4_KIBIBYTE_TIER_MEBIBYTES_PER_SECOND;
    append_output_line(emitter.emit_raw(&format!("name=poscontrol arm=e6_single_core mibs={single_core_mebibytes_per_second:.2} e6={E6_AES_4_KIBIBYTE_TIER_MEBIBYTES_PER_SECOND} dev={:.1}% ok={}", deviation_from_e6*100.0, deviation_from_e6 <= 0.20)));
    if deviation_from_e6 > 0.20 {
        flush_output_and_abort_round(&mut output_text, &mut emitter, &format!("单核 {single_core_mebibytes_per_second:.0} MiB/s 与 E6 的 {E6_AES_4_KIBIBYTE_TIER_MEBIBYTES_PER_SECOND} 差 {:.0}%（>20%）—— 本 harness 与 E6 测的不是同一件事，整轮作废", deviation_from_e6*100.0));
    }

    // ── 阳性对照 2：算力受限臂必须接近线性扩展 ──
    let one_thread_nanoseconds = (0..3).map(|_| compute(1, 800_000_000)).min().unwrap();
    let sixteen_thread_nanoseconds = (0..3).map(|_| compute(16, 800_000_000)).min().unwrap();
    let compute_speedup = one_thread_nanoseconds as f64 / sixteen_thread_nanoseconds as f64;
    append_output_line(emitter.emit_raw(&format!("name=poscontrol arm=compute threads=16 speedup={compute_speedup:.2} ok={}", compute_speedup >= 4.0)));
    if compute_speedup < 4.0 { flush_output_and_abort_round(&mut output_text, &mut emitter, &format!("算力受限臂 16 线程只加速 {compute_speedup:.2}×（要求 ≥4）—— harness 看不见并行度，整轮作废")); }

    // ── 主体：两种算法 × 两档工作集 × 线程数扫描 ──
    for (workset_label, per_thread_mebibytes) in [("l2_resident", 1usize), ("dram", 64usize)] {
        for cipher_algorithm in [CipherAlgorithm::Aes, CipherAlgorithm::Chacha] {
            let mut one_thread_mebibytes_per_second = 0.0f64;
            for &thread_count in &[1usize, 2, 4, 8, 16, 32] {
                if thread_count > core_count { continue; }
                let mut best_nanoseconds = u64::MAX; let mut best_byte_count = 0;
                for _ in 0..round_count { let (elapsed_nanoseconds, sealed_byte_count) = time_parallel_seal_in_nanoseconds(cipher_algorithm, thread_count, per_thread_mebibytes); if elapsed_nanoseconds < best_nanoseconds { best_nanoseconds = elapsed_nanoseconds; best_byte_count = sealed_byte_count; } }
                let measured_mebibytes_per_second = throughput_in_mebibytes_per_second(best_byte_count, best_nanoseconds);
                if thread_count == 1 { one_thread_mebibytes_per_second = measured_mebibytes_per_second; }
                append_output_line(emitter.emit_raw(&format!(
                    "name=scale workset={workset_label} per_thread_mib={per_thread_mebibytes} alg={} threads={thread_count} mibs={measured_mebibytes_per_second:.1} gbps={:.2} speedup={:.2}",
                    cipher_algorithm.name(), measured_mebibytes_per_second * 1024.0 * 1024.0 / 1e9, measured_mebibytes_per_second / one_thread_mebibytes_per_second)));
            }
        }
    }
    append_output_line(emitter.finish());
    print!("{output_text}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 加密必须真的改变缓冲内容——否则量到的是一个空循环。
    #[test]
    fn seal_actually_transforms_the_buffer() {
        for cipher_algorithm in [CipherAlgorithm::Aes, CipherAlgorithm::Chacha] {
            let mut buffer = vec![0u8; ENCRYPTION_UNIT_BYTES * 4];
            let buffer_before_sealing = buffer.clone();
            let sealed_byte_count = seal_buffer_in_place(cipher_algorithm, &mut buffer);
            assert_eq!(sealed_byte_count, ENCRYPTION_UNIT_BYTES * 4);
            assert_ne!(buffer, buffer_before_sealing, "{} 没有改变缓冲，测的是空循环", cipher_algorithm.name());
        }
    }

    /// 两种算法必须产出不同的密文——否则枚举分派串了。
    #[test]
    fn the_two_algorithms_differ() {
        let (mut aes_buffer, mut chacha_buffer) = (vec![0u8; ENCRYPTION_UNIT_BYTES], vec![0u8; ENCRYPTION_UNIT_BYTES]);
        seal_buffer_in_place(CipherAlgorithm::Aes, &mut aes_buffer); seal_buffer_in_place(CipherAlgorithm::Chacha, &mut chacha_buffer);
        assert_ne!(aes_buffer, chacha_buffer, "两种算法产出相同密文，分派串了");
    }

    /// 每个 UNIT 都要被加密到，不是只加密第一块。
    #[test]
    fn every_unit_is_covered() {
        let mut buffer = vec![0u8; ENCRYPTION_UNIT_BYTES * 3];
        seal_buffer_in_place(CipherAlgorithm::Aes, &mut buffer);
        for (unit_index, unit_chunk) in buffer.chunks(ENCRYPTION_UNIT_BYTES).enumerate() {
            assert!(unit_chunk.iter().any(|&byte_value| byte_value != 0), "第 {unit_index} 个单元没被加密");
        }
    }

    /// 算力受限对照必须固定总工作量：线程数翻倍，单线程迭代数减半。
    #[test]
    fn compute_control_holds_total_work_constant() {
        // 用极小的总量，只验分派逻辑不验性能
        let one_thread_nanoseconds = compute(1, 1_000_000);
        let four_thread_nanoseconds = compute(4, 1_000_000);
        assert!(one_thread_nanoseconds > 0 && four_thread_nanoseconds > 0);
        assert!(four_thread_nanoseconds < one_thread_nanoseconds, "4 线程做同样总量应当更快，实测 {four_thread_nanoseconds} vs {one_thread_nanoseconds}");
    }
}
