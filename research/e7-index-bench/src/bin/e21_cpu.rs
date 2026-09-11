//! E21 的 **CPU 基线**：一次带宽受限的扫描能跑多快。
//!
//! 它要回答的只有一件事：[experiments.md] E21 判据 1 里的分子。
//! GPU 路径的地板已由传输段给出（scrub 走单程、EC 重建走来回），
//! **CPU 侧只要超过那个地板，对应那一格就不必等核函数写出来就已经死了。**
//!
//! 测的是 XOR 折叠（parity / scrub 内层循环的代理），不是压缩也不是加密——
//! ⚠️ **拿它去谈压缩或 AEAD 是误用**，那两样的 CPU 侧另有其数（E6 测过加密）。
//!
//! **阳性对照必须用另一条臂，不能拿被测臂自己当对照。**
//! 被测臂（XOR 折叠）是**带宽受限**的：一个核就能吃满 DDR5 的大部分带宽，
//! 所以它**本来就不该线性扩展**——拿「16 线程要快 2 倍」去要求它，
//! 等于把一个合法结果判成 harness 坏了（本实验第一版正是如此，实测 1.16×）。
//! ⇒ 对照另设一条**算力受限**的臂：数据只有 L1 那么大、每个元素做一条长依赖链，
//! 它必须接近线性扩展。它测不出加速才说明 harness 看不见并行度，那时才整轮作废。

use e7_index_bench::Emitter;
use std::sync::Arc;
use std::time::Instant;

/// XOR 折叠一段字节，返回校验值。`black_box` 防止整段被优化掉。
fn xor_fold_words(words: &[u64]) -> u64 {
    let mut folded_checksum = 0u64;
    for &word in words {
        folded_checksum ^= word;
    }
    std::hint::black_box(folded_checksum)
}

/// 第 `thread_index` 个线程负责 `[chunk_start, chunk_end)`。**生产路径与测试必须共用这一个函数**——
/// 测试里再写一遍分块，就测不到生产路径的分块（变异测试当场证明过这一点）。
fn thread_chunk_bounds(element_count: usize, thread_count: usize, thread_index: usize) -> (usize, usize) {
    let chunk_length = element_count.div_ceil(thread_count);
    ((thread_index * chunk_length).min(element_count), ((thread_index + 1) * chunk_length).min(element_count))
}

fn time_xor_scan_in_nanoseconds(shared_words: &Arc<Vec<u64>>, thread_count: usize) -> u64 {
    let word_count = shared_words.len();
    let start_instant = Instant::now();
    let mut worker_handles = Vec::with_capacity(thread_count);
    for thread_index in 0..thread_count {
        let shared_words_for_thread = Arc::clone(shared_words);
        worker_handles.push(std::thread::spawn(move || {
            let (chunk_start, chunk_end) = thread_chunk_bounds(word_count, thread_count, thread_index);
            xor_fold_words(&shared_words_for_thread[chunk_start..chunk_end])
        }));
    }
    let mut combined_checksum = 0u64;
    for worker_handle in worker_handles {
        combined_checksum ^= worker_handle.join().unwrap();
    }
    std::hint::black_box(combined_checksum);
    start_instant.elapsed().as_nanos() as u64
}

/// **阳性对照臂：算力受限。** 每线程做一条长依赖链，内存带宽完全不是瓶颈。
///
/// ⚠️ `total_iterations` 是**总**迭代次数，按线程数均分——**必须固定总工作量**。
/// 让每条线程都做 `total_iterations` 次的话测的是弱扩展：完美并行时挂钟持平、比值 ≈ 1，
/// 而那会被「要求比值 ≥ 4」判成不并行。本实验第二版正是如此，实测 0.96。
fn time_compute_bound_chain_in_nanoseconds(thread_count: usize, total_iterations: u64) -> u64 {
    let iterations_per_thread = total_iterations / thread_count as u64;
    let start_instant = Instant::now();
    let mut worker_handles = Vec::with_capacity(thread_count);
    for thread_index in 0..thread_count {
        worker_handles.push(std::thread::spawn(move || {
            let mut chain_state = thread_index as u64 | 1;
            let iterations_per_thread = iterations_per_thread;
            for _ in 0..iterations_per_thread {
                // 依赖链：每一步都要上一步的结果，塞不进并行的执行单元
                chain_state = chain_state.wrapping_mul(0x9E37_79B9_7F4A_7C15).rotate_left(23) ^ 0x5851_F42D_4C95_7F2D;
            }
            chain_state
        }));
    }
    let mut combined_chain_state = 0u64;
    for worker_handle in worker_handles { combined_chain_state ^= worker_handle.join().unwrap(); }
    std::hint::black_box(combined_chain_state);
    start_instant.elapsed().as_nanos() as u64
}

fn decimal_gigabytes_per_second(bytes: usize, nanoseconds: u64) -> f64 {
    if nanoseconds == 0 {
        return f64::NAN;
    }
    bytes as f64 / (nanoseconds as f64)
}

fn main() {
    let buffer_mebibytes: usize = std::env::args().nth(1).and_then(|argument| argument.parse().ok()).unwrap_or(2048);
    let round_count: usize = std::env::args().nth(2).and_then(|argument| argument.parse().ok()).unwrap_or(5);
    let core_count = std::thread::available_parallelism().map(|parallelism| parallelism.get()).unwrap_or(1);
    let mut emitter = Emitter::new();
    let mut output_text = String::new();
    let mut append_output_line = |output_line: String| { output_text.push_str(&output_line); output_text.push('\n'); };

    let word_count = buffer_mebibytes * 1024 * 1024 / 8;
    let shared_words = Arc::new((0..word_count).map(|word_index| word_index as u64).collect::<Vec<u64>>());
    let buffer_byte_count = word_count * 8;
    append_output_line(emitter.emit_raw(&format!("name=config mib={buffer_mebibytes} rounds={round_count} cores={core_count}")));

    let mut best_nanoseconds_by_thread_count = std::collections::BTreeMap::new();
    for &thread_count in &[1usize, 2, 4, 8, 16, 32] {
        if thread_count > core_count * 2 { continue; }
        let mut best_nanoseconds = u64::MAX;
        for _ in 0..round_count { best_nanoseconds = best_nanoseconds.min(time_xor_scan_in_nanoseconds(&shared_words, thread_count)); }
        best_nanoseconds_by_thread_count.insert(thread_count, best_nanoseconds);
        append_output_line(emitter.emit_raw(&format!(
            "name=scan threads={thread_count} best_ns={best_nanoseconds} gbps={:.2} speedup={:.2}",
            decimal_gigabytes_per_second(buffer_byte_count, best_nanoseconds), best_nanoseconds_by_thread_count[&1] as f64 / best_nanoseconds as f64
        )));
    }

    // 被测臂的扩展比：**如实报告，不作判据**——带宽受限的东西本来就不线性扩展
    append_output_line(emitter.emit_raw(&format!(
        "name=scaling arm=bandwidth threads16_speedup={:.2} peak_gbps={:.2}",
        best_nanoseconds_by_thread_count[&1] as f64 / best_nanoseconds_by_thread_count[&16] as f64,
        best_nanoseconds_by_thread_count.values().map(|&thread_count_best_nanoseconds| decimal_gigabytes_per_second(buffer_byte_count, thread_count_best_nanoseconds)).fold(0.0, f64::max)
    )));

    // ── 阳性对照：算力受限的臂必须接近线性扩展 ──
    let total_iterations = 800_000_000u64; // 总迭代次数，按线程均分
    let one_thread_nanoseconds = (0..3).map(|_| time_compute_bound_chain_in_nanoseconds(1, total_iterations)).min().unwrap();
    let sixteen_thread_nanoseconds = (0..3).map(|_| time_compute_bound_chain_in_nanoseconds(16, total_iterations)).min().unwrap();
    let compute_speedup = one_thread_nanoseconds as f64 / sixteen_thread_nanoseconds as f64;
    append_output_line(emitter.emit_raw(&format!(
        "name=poscontrol arm=compute threads=16 t1_ns={one_thread_nanoseconds} t16_ns={sixteen_thread_nanoseconds} speedup={compute_speedup:.2}"
    )));
    if compute_speedup < 4.0 {
        append_output_line(emitter.finish());
        print!("{output_text}");
        eprintln!("E21CPU: 算力受限臂 16 线程只加速 {compute_speedup:.2}×（要求 ≥4）—— harness 看不见并行度，整轮作废");
        std::process::exit(4);
    }
    append_output_line(emitter.finish());
    print!("{output_text}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 折叠结果必须与顺序无关且非平凡——被优化掉的话它会恒为 0。
    /// **把吞吐的单位钉死成十进制 GB/s。** 这是本实验唯一的绝对值断言。
    ///
    /// ⚠️ **为什么单靠单位就值一条测试**（2026-08-29 对抗验证补入）：
    /// 本实验此前**一条绝对值断言都没有**——两个单测钉的是「折叠与顺序无关」
    /// 与「分块是一个划分」，都是**结构性质**，量出来的那个数没有任何东西钉它。
    ///
    /// 而这个数要跨语言比：CPU 侧是本文件（Rust），GPU 侧是
    /// `research/scripts/e21-transfer.py`（Python，`gb = n / 1e9`）。
    /// **两侧任何一侧改成 GiB（2^30），比值就偏 7.4%**——
    /// 而 decisions.md D24（后台重活能不能卸给 GPU） 的判决恰恰落在这个量级上
    /// （CPU 64.91 对 GPU 端到端 50.3，净收益 0.78×）。
    /// ⇒ 这正是 `test-discipline.md`「只让多条臂互相比，测不出所有臂一起错」的跨实验形态：
    /// **两侧一起换单位，比值仍然「成立」。**
    #[test]
    fn throughput_unit_is_decimal_gigabytes_per_second() {
        // 10^9 字节 / 10^9 纳秒 = 1 GB/s，按定义
        assert_eq!(decimal_gigabytes_per_second(1_000_000_000, 1_000_000_000), 1.0,
                   "单位不是十进制 GB/s——GPU 侧的 e21-transfer.py 用的是 n / 1e9，两侧必须同口径");
        // 2 GiB / 1 s 应当是 2.147… GB/s，不是 2.0：若这里等于 2.0 就说明用了 GiB
        let two_gibibytes = 2usize * 1024 * 1024 * 1024;
        let gigabytes_per_second = decimal_gigabytes_per_second(two_gibibytes, 1_000_000_000);
        assert!((gigabytes_per_second - 2.147_483_648).abs() < 1e-6,
                "2 GiB/s 算成了 {gigabytes_per_second}，说明分母或分子用了 2^30 而不是 10^9");
    }

    #[test]
    fn fold_is_nontrivial_and_order_independent() {
        let words: Vec<u64> = (1..=1000).collect();
        let forward_fold = xor_fold_words(&words);
        let mut reversed_words = words.clone();
        reversed_words.reverse();
        assert_eq!(forward_fold, xor_fold_words(&reversed_words), "XOR 折叠应当与顺序无关");
        assert_ne!(forward_fold, 0, "折叠恒为 0 说明它被优化掉了，量到的就不是内存带宽");
    }

    /// 分块后各线程的结果异或起来，必须等于整段折叠——否则多线程臂算的不是同一件事。
    #[test]
    fn chunked_fold_equals_whole() {
        let words: Vec<u64> = (0..100_000).map(|word_index| word_index as u64 * 7 + 3).collect();
        let whole_fold = xor_fold_words(&words);
        for thread_count in [2usize, 3, 8] {
            let mut combined_fold = 0u64;
            for thread_index in 0..thread_count {
                let (chunk_start, chunk_end) = thread_chunk_bounds(words.len(), thread_count, thread_index);
                combined_fold ^= xor_fold_words(&words[chunk_start..chunk_end]);
            }
            assert_eq!(combined_fold, whole_fold, "{thread_count} 分块的结果与整段不等");
        }
        // 分块必须**恰好覆盖**整段：首块从 0 起、末块到 n 止、块块首尾相接
        for thread_count in [1usize, 2, 5, 16] {
            let element_count = 1000;
            let mut previous_chunk_end = 0;
            for thread_index in 0..thread_count {
                let (chunk_start, chunk_end) = thread_chunk_bounds(element_count, thread_count, thread_index);
                assert_eq!(chunk_start, previous_chunk_end, "th={thread_count} t={thread_index} 有空隙或重叠");
                previous_chunk_end = chunk_end;
            }
            assert_eq!(previous_chunk_end, element_count, "th={thread_count} 分块没覆盖到末尾");
        }
    }
}
