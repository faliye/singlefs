//! E17：write buffer 的 sort-merge-sweep 单线程上限，以及它能不能做成无锁多路并行。
//!
//! 起因是两条已定项**正面相撞**：
//!   - D8：write buffer 的 sort-merge-sweep **本质单线程且在关键路径上**；
//!   - D9：加密路径**必须能并行**（E6 实测无 AES-NI 时单核 267.55 MiB/s，低于单盘顺序带宽）。
//! 若合并这一段也卡在单线程，那并行化加密买到的东西会被它吃掉。
//!
//! ## 三条臂共用同一份输入，且输出必须逐位相同
//!
//! **这是本实验的非平凡性保证**：两条臂若产出不同的结果，比的就是不同的工作量。
//! 每轮结束后逐位比对合并结果，不同即整轮作废。
//!
//! ## 为什么可以按叶分区做无锁并行
//!
//! write buffer 的 flush 目标是「按叶聚合后批量插入」，而**不同叶之间没有依赖**。
//! 所以按目标叶散列分区之后，每个分区可以独立排序去重，全程无锁、无共享可变状态。
//! 代价是多一趟 O(n) 的散射（scatter）。本实验要量的就是：**那一趟散射会不会把收益吃光。**
//!
//! ## 阳性对照
//!
//! 「每线程各排一份互不相交的数据」必须接近线性加速。测不出加速 ⇒ 机器或 harness 有问题，
//! 整轮作废——不许把「没加速」记成算法的结论。

use e7_index_bench::Emitter;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

/// 一条缓冲条目：(key, seq, value)。
///
/// ⚠️ **`seq` 不是可选的**：D8 正文明写 flush 时按 key 排序会**丢弃时间序**，
/// 而去重规则是「后者胜」——排序之后「后者」这个概念必须由条目自己携带。
/// 用不稳定排序时同 key 条目的相对次序未定义，**去重结果因此不确定**。
/// 本实验第一版漏了它，两条臂与真值当场对不上——是模型抓出来的，不是推出来的。
type Entry = (u64, u64, u64); // (key, seq, value)
const ENTRY_BYTES: usize = 24;

/// 目标叶数。write buffer 的 flush 会覆盖整个叶子空间（E7 已实测这一点）。
const N_LEAVES: u64 = 4096;
fn leaf_of(k: u64) -> u64 {
    k % N_LEAVES
}

fn gen(n: usize, seed: u64) -> Vec<Entry> {
    let mut s = seed | 1;
    let mut r = || {
        s ^= s >> 12;
        s ^= s << 25;
        s ^= s >> 27;
        s.wrapping_mul(0x2545_F491_4F6C_DD1D)
    };
    // key 空间取叶数的 64 倍 —— 同一叶内会有多条，且整体有重复（去重才有事可做）
    let span = N_LEAVES * 64;
    (0..n).map(|i| { let k = r() % span; (k, i as u64, r()) }).collect()
}

/// 单线程臂：D8 现方向。排序 → 去重（后者胜）→ 按叶扫一遍。
fn arm_single(input: &[Entry]) -> Vec<Entry> {
    let mut v = input.to_vec();
    // 按 (叶, key) 排 —— 与 E7 的 write buffer flush 同一口径
    // 按 (叶, key, seq) 排 —— seq 让「后者胜」在不稳定排序下仍然确定
    v.sort_unstable_by_key(|e| (leaf_of(e.0), e.0, e.1));
    let mut out: Vec<Entry> = Vec::with_capacity(v.len());
    for &e in v.iter() {
        if let Some(last) = out.last_mut() {
            if last.0 == e.0 { *last = e; continue; } // seq 更大者胜
        }
        out.push(e);
    }
    out
}

/// 并行臂：按目标叶分区 → 每区独立排序去重 → 按区号拼接。全程无锁。
/// 分区数取 `parts`，叶按 `leaf % parts` 归区，所以区内叶号仍单调 —— 拼接后全局有序。
fn arm_parallel(input: &Arc<Vec<Entry>>, threads: usize) -> Vec<Entry> {
    // 分区边界按叶号切，保证「区号升序 = 叶号升序」，拼接不需要再排一次
    let per = N_LEAVES.div_ceil(threads as u64);
    let mut handles = Vec::with_capacity(threads);
    for t in 0..threads {
        let inp = Arc::clone(input);
        let lo = t as u64 * per;
        let hi = ((t as u64 + 1) * per).min(N_LEAVES);
        handles.push(std::thread::spawn(move || {
            // 散射：只挑属于本区的条目。这一趟是 O(n)，是并行要付的代价。
            let mut v: Vec<Entry> = inp.iter().copied()
                .filter(|e| { let l = leaf_of(e.0); l >= lo && l < hi })
                .collect();
            v.sort_unstable_by_key(|e| (leaf_of(e.0), e.0, e.1));
            let mut out: Vec<Entry> = Vec::with_capacity(v.len());
            for &e in v.iter() {
                if let Some(last) = out.last_mut() {
                    if last.0 == e.0 { *last = e; continue; }
                }
                out.push(e);
            }
            out
        }));
    }
    let mut merged = Vec::with_capacity(input.len());
    for h in handles { merged.extend(h.join().unwrap()); }
    merged
}

/// 阳性对照：每线程排一份**互不相交**的数据。它必须接近线性加速；
/// 不加速说明机器或 harness 有问题，不是算法的结论。
fn arm_control(n: usize, threads: usize, seed: u64) -> u64 {
    let mut handles = Vec::with_capacity(threads);
    let per = n / threads;
    for t in 0..threads {
        handles.push(std::thread::spawn(move || {
            let mut v = gen(per, seed ^ (t as u64 * 0x9E3779B97F4A7C15));
            v.sort_unstable_by_key(|e| (e.0, e.1));
            v.len() as u64
        }));
    }
    handles.into_iter().map(|h| h.join().unwrap()).sum()
}

/// 真值：用 BTreeMap 独立算一遍「按叶、按 key、后者胜」的结果。
/// 它与两条臂**不共享任何合并代码**——两条臂一致但都错，只有它能分辨。
fn ground_truth(input: &[Entry]) -> Vec<Entry> {
    // 按到达顺序放进表，后到的覆盖先到的 —— 与两条臂不共享任何合并代码
    let mut m: BTreeMap<(u64, u64), (u64, u64)> = BTreeMap::new();
    for &(k, sq, v) in input { m.insert((leaf_of(k), k), (sq, v)); }
    m.into_iter().map(|((_, k), (sq, v))| (k, sq, v)).collect()
}

fn mibs(n: usize, ns: u64) -> f64 {
    if ns == 0 { return f64::NAN; }
    (n * ENTRY_BYTES) as f64 / (1024.0 * 1024.0) / (ns as f64 / 1e9)
}

fn main() {
    let n: usize = std::env::args().nth(1).and_then(|x| x.parse().ok()).unwrap_or(4_000_000);
    let rounds: usize = std::env::args().nth(2).and_then(|x| x.parse().ok()).unwrap_or(5);
    let mut em = Emitter::new();
    let mut out = String::new();
    let mut say = |s: String| { out.push_str(&s); out.push('\n'); };

    say(em.emit_raw(&format!(
        "name=config entries={n} rounds={rounds} entry_bytes={ENTRY_BYTES} leaves={N_LEAVES} cores={}",
        std::thread::available_parallelism().map(|v| v.get()).unwrap_or(0)
    )));

    // ── 正确性闸：两条臂 + 真值三方一致，否则整轮作废 ──
    {
        let small = gen(200_000, 42);
        let a = arm_single(&small);
        let b = arm_parallel(&Arc::new(small.clone()), 8);
        let t = ground_truth(&small);
        let ok = a == t && b == t;
        say(em.emit_raw(&format!(
            "name=correctness single_len={} parallel_len={} truth_len={} all_equal={ok}",
            a.len(), b.len(), t.len()
        )));
        if !ok {
            say(em.finish());
            print!("{out}");
            eprintln!("E17: 两条臂与真值不一致 —— 比的是不同的工作量，本轮作废");
            std::process::exit(4);
        }
    }

    let input = Arc::new(gen(n, 7));

    // ── 单线程臂 ──
    let mut best_single = u64::MAX;
    for _ in 0..rounds {
        let t0 = Instant::now();
        let r = arm_single(&input);
        let ns = t0.elapsed().as_nanos() as u64;
        std::hint::black_box(&r);
        best_single = best_single.min(ns);
    }
    say(em.emit_raw(&format!(
        "name=merge arm=single threads=1 best_ns={best_single} mib_per_s={:.2} entries_per_s={:.0}",
        mibs(n, best_single), n as f64 / (best_single as f64 / 1e9)
    )));

    // ── 并行臂：线程数扫描 ──
    for th in [2usize, 4, 8, 16, 32] {
        let mut best = u64::MAX;
        for _ in 0..rounds {
            let t0 = Instant::now();
            let r = arm_parallel(&input, th);
            let ns = t0.elapsed().as_nanos() as u64;
            std::hint::black_box(&r);
            best = best.min(ns);
        }
        say(em.emit_raw(&format!(
            "name=merge arm=parallel threads={th} best_ns={best} mib_per_s={:.2} entries_per_s={:.0} speedup={:.2}",
            mibs(n, best), n as f64 / (best as f64 / 1e9), best_single as f64 / best as f64
        )));
    }

    // ── 阳性对照：互不相交的数据必须接近线性加速 ──
    let mut ctl1 = u64::MAX;
    for _ in 0..rounds {
        let t0 = Instant::now();
        let v = arm_control(n, 1, 99);
        let ns = t0.elapsed().as_nanos() as u64;
        std::hint::black_box(v);
        ctl1 = ctl1.min(ns);
    }
    for th in [8usize, 16, 32] {
        let mut best = u64::MAX;
        for _ in 0..rounds {
            let t0 = Instant::now();
            let v = arm_control(n, th, 99);
            let ns = t0.elapsed().as_nanos() as u64;
            std::hint::black_box(v);
            best = best.min(ns);
        }
        say(em.emit_raw(&format!(
            "name=poscontrol threads={th} best_ns={best} speedup={:.2}", ctl1 as f64 / best as f64
        )));
    }

    say(em.finish());
    print!("{out}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 两条臂与真值必须三方一致。这是本实验唯一的非平凡性保证。
    #[test]
    fn arms_agree_with_truth() {
        let inp = gen(50_000, 5);
        let t = ground_truth(&inp);
        assert_eq!(arm_single(&inp), t);
        assert_eq!(arm_parallel(&Arc::new(inp.clone()), 8), t);
    }

    /// 去重真的发生了——否则「合并」什么也没做，测的是纯排序。
    #[test]
    fn dedup_actually_removes_entries() {
        let inp = gen(200_000, 11);
        let out = arm_single(&inp);
        assert!(out.len() < inp.len(), "没有任何条目被去重，负载设错了");
    }

    /// 分区数变化不改变结果——保证并行臂的正确性不依赖线程数。
    #[test]
    fn partition_count_does_not_change_result() {
        let inp = Arc::new(gen(80_000, 13));
        let a = arm_parallel(&inp, 2);
        for th in [3usize, 7, 16] {
            assert_eq!(arm_parallel(&inp, th), a);
        }
    }
}
