//! E14：I-3.1 的判别力——幂等完整值 vs 增量 Δ。
//!
//! 测判别力不是性能：注入 4 类语义 bug，看 I-3.1 在两种条目形态下各抓到几类。
//! **不需要设备也不需要虚机**——它是逻辑实验，放进虚机跑是做样子。
//!
//! 三方，不是两方：运行时计数器 / checker 重算 / 真值。
//! I-3.1 只比前两者；真值用来分辨「两边一起错」——盲区正在那里。
//!
//! 建模的要害：运行时的账在**条目入缓冲时**增量维护（fs-design.md「记账是事务的副产品」），
//! 树在 **flush 合并时**更新。两个时刻、两段代码——所以 flush 侧的 bug 能让两者对不上。
//! 若把账也从合并结果导出，I-3.1 就成了恒等式，这实验什么都测不到。
//!
//! 形态 B 的前提（这是 D11 代价 1 的**命题**，不是本实验的发现）：
//! 账本身就是权威内容，checker 没有可独立遍历的东西，只能拿同一个累加函数重放 Δ 日志。
//! 形态同 bcachefs `disk_accounting.h:208 this_cpu_add(e->v[gc][i], a.v->d[i])`——
//! 运行时与 GC 重建只差一个下标。

use e7_index_bench::Emitter;
use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq)]
enum Form { Idempotent, Delta }

#[derive(Clone, Copy, PartialEq, Debug)]
enum Bug { None, MergeReversed, TombstoneIgnored, DropMessage, DoubleApply }

const N_KEYS: u64 = 512;
const BATCH: usize = 64;

#[derive(Clone, Copy)]
enum Op { Set(u64, i64), Del(u64) }

fn gen_ops(n: usize, seed: u64) -> Vec<Op> {
    let mut s = seed | 1;
    let mut r = || { s ^= s >> 12; s ^= s << 25; s ^= s >> 27; s.wrapping_mul(0x2545F4914F6CDD1D) };
    (0..n).map(|_| {
        let k = r() % N_KEYS;
        if r() % 4 == 0 { Op::Del(k) } else { Op::Set(k, (r() % 1000 + 1) as i64) }
    }).collect()
}

/// 真值：按顺序放进一张表求和。没有合并规则可以写错。
fn ground_truth(ops: &[Op]) -> i64 {
    let mut m: BTreeMap<u64, i64> = BTreeMap::new();
    for op in ops {
        match *op { Op::Set(k, v) => { m.insert(k, v); } Op::Del(k) => { m.remove(&k); } }
    }
    m.values().sum()
}

/// 一轮的三个读数。
struct Reading { rt: i64, ck: i64, truth: i64, entries: usize }

// ───────── 形态 A：幂等完整值 (key, Some(size)) / (key, None)=tombstone ─────────
// 权威是内容。账是缓存。checker 遍历内容求和——**独立实现，不碰下面的合并函数**。

fn flush_idem(buf: &[(u64, Option<i64>)], tree: &mut BTreeMap<u64, i64>, bug: Bug) {
    // 合并（后者胜）后落树。bug 注在这里 = 注在 flush 侧。
    let mut merged: BTreeMap<u64, Option<i64>> = BTreeMap::new();
    for &(k, v) in buf {
        match bug {
            Bug::MergeReversed => { merged.entry(k).or_insert(v); }        // 取先者胜
            Bug::TombstoneIgnored => { if v.is_some() { merged.insert(k, v); } } // 墓碑当普通值
            _ => { merged.insert(k, v); }
        }
    }
    for (&k, &v) in &merged {
        match v { Some(s) => { tree.insert(k, s); } None => { tree.remove(&k); } }
    }
}

fn run_idem(ops: &[Op], bug: Bug) -> Reading {
    let mut tree: BTreeMap<u64, i64> = BTreeMap::new();
    let mut shadow: BTreeMap<u64, i64> = BTreeMap::new(); // 运行时对「当前值」的认识
    let mut rt: i64 = 0;
    let mut entries = 0usize;
    for chunk in ops.chunks(BATCH) {
        let mut buf: Vec<(u64, Option<i64>)> = Vec::with_capacity(chunk.len());
        for op in chunk {
            // 入缓冲的同时增量记账——这一步与 flush 无关，是事务提交侧的代码
            let (k, nv) = match *op { Op::Set(k, v) => (k, Some(v)), Op::Del(k) => (k, None) };
            let old = shadow.get(&k).copied().unwrap_or(0);
            let new = nv.unwrap_or(0);
            rt += new - old;
            match nv { Some(s) => { shadow.insert(k, s); } None => { shadow.remove(&k); } }
            buf.push((k, nv));
        }
        if bug == Bug::DropMessage && !buf.is_empty() { buf.pop(); }
        if bug == Bug::DoubleApply { let d = buf.clone(); buf.extend(d); }
        entries += buf.len();
        flush_idem(&buf, &mut tree, bug);
    }
    Reading { rt, ck: tree.values().sum(), truth: ground_truth(ops), entries }
}

// ───────── 形态 B：增量 Δ (key, Δbytes)。合并 = 累加。 ─────────

/// 运行时与 checker **共用的同一个累加函数**。bug 注在这里，于是两侧一起错。
fn accumulate(acc: &mut i64, d: i64, bug: Bug) {
    match bug { Bug::MergeReversed => *acc -= d, _ => *acc += d }
}

fn run_delta(ops: &[Op], bug: Bug) -> Reading {
    let mut shadow: BTreeMap<u64, i64> = BTreeMap::new();
    let mut rt: i64 = 0;
    let mut log: Vec<(u64, i64)> = Vec::new(); // 持久化下来的那份 Δ 日志
    for chunk in ops.chunks(BATCH) {
        let mut buf: Vec<(u64, i64)> = Vec::with_capacity(chunk.len());
        for op in chunk {
            let (k, d) = match *op {
                Op::Set(k, v) => { let old = shadow.get(&k).copied().unwrap_or(0);
                                   shadow.insert(k, v); (k, v - old) }
                Op::Del(k) => { let old = shadow.get(&k).copied().unwrap_or(0);
                                if bug == Bug::TombstoneIgnored { (k, 0) }   // 删除不产生负 Δ
                                else { shadow.remove(&k); (k, -old) } }
            };
            buf.push((k, d));
        }
        if bug == Bug::DropMessage && !buf.is_empty() { buf.pop(); }
        if bug == Bug::DoubleApply { let d = buf.clone(); buf.extend(d); }
        for &(_, d) in &buf { accumulate(&mut rt, d, bug); }
        log.extend(buf);
    }
    // checker：重放同一份日志，用同一个 accumulate
    let mut ck = 0i64;
    for &(_, d) in &log { accumulate(&mut ck, d, bug); }
    Reading { rt, ck, truth: ground_truth(ops), entries: log.len() }
}

fn main() {
    let mut em = Emitter::new();
    let seeds: [u64; 5] = [11, 22, 33, 44, 55];
    let ops_n = 20_000usize;
    println!("{}", em.emit_raw(&format!(
        "name=config keys={N_KEYS} ops={ops_n} batch={BATCH} seeds={}", seeds.len())));
    for form in [Form::Idempotent, Form::Delta] {
        let fname = if form == Form::Idempotent { "idempotent" } else { "delta" };
        for bug in [Bug::None, Bug::MergeReversed, Bug::TombstoneIgnored, Bug::DropMessage, Bug::DoubleApply] {
            let (mut red, mut correct, mut ent) = (0u32, 0u32, 0usize);
            for sd in seeds {
                let ops = gen_ops(ops_n, sd);
                let r = if form == Form::Idempotent { run_idem(&ops, bug) } else { run_delta(&ops, bug) };
                if r.rt != r.ck { red += 1; }
                if r.rt == r.truth { correct += 1; }
                ent = r.entries; // 各 seed 相同：条目数只由 ops_n/BATCH 与注入决定
            }
            println!("{}", em.emit_raw(&format!(
                "name=case form={fname} bug={bug:?} i31_red={red}/{n} value_correct={correct}/{n} flushed_entries={ent}",
                n = seeds.len())));
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **真值必须是独立算出来的**，不许走被测的那两条合并路径。
    /// 它是本实验唯一的裁判——它错了，四类注入的判定全部作废。
    #[test]
    fn ground_truth_is_a_plain_replay() {
        let ops = vec![Op::Set(1, 10), Op::Set(2, 20), Op::Set(1, 5), Op::Del(2)];
        assert_eq!(ground_truth(&ops), 5, "真值应当只剩 key1=5");
        assert_eq!(ground_truth(&[]), 0);
        // 删一个不存在的 key 不该改变结果
        assert_eq!(ground_truth(&[Op::Set(1, 7), Op::Del(9)]), 7);
    }

    /// **不注入 bug 时两种形态都必须算对账**，否则「注入后算错」就说明不了任何事。
    #[test]
    fn both_forms_are_correct_without_any_bug() {
        let ops = gen_ops(2000, 11);
        for (name, r) in [("幂等", run_idem(&ops, Bug::None)), ("增量", run_delta(&ops, Bug::None))] {
            assert_eq!(r.rt, r.truth, "{name}形态在无 bug 时运行时账就不对");
            assert_eq!(r.ck, r.truth, "{name}形态在无 bug 时 checker 账就不对");
        }
    }

    /// **四类注入必须真的把账算错**——否则「I-3.1 抓不到」是因为根本没有错可抓，
    /// 那样得出的「判别力为零」是伪的。这条是本实验非平凡性的根。
    #[test]
    fn every_injected_bug_actually_corrupts_the_accounting() {
        let ops = gen_ops(2000, 13);
        for bug in [Bug::MergeReversed, Bug::TombstoneIgnored, Bug::DropMessage, Bug::DoubleApply] {
            let d = run_delta(&ops, bug);
            assert_ne!(d.rt, d.truth, "增量形态注入 {bug:?} 之后账居然还是对的");
        }
    }

    /// **I-3.1 的判别力就是 `rt == ck` 这一个比较**：
    /// 增量形态下两者同源 ⇒ 永远相等 ⇒ 判别力为零。
    /// 这条把「为什么抓不到」钉死在机制上，不是钉在某一次的数字上。
    #[test]
    fn delta_form_makes_the_invariant_blind_by_construction() {
        let ops = gen_ops(2000, 17);
        for bug in [Bug::None, Bug::MergeReversed, Bug::TombstoneIgnored, Bug::DropMessage, Bug::DoubleApply] {
            let d = run_delta(&ops, bug);
            assert_eq!(d.rt, d.ck, "增量形态下 I-3.1 竟然分开了 {bug:?}——那它就不是同源的了");
        }
    }

    /// 幂等形态下 I-3.1 必须抓到三类真 bug，且对 `DoubleApply` 不报警
    /// （那一类在幂等形态下根本不是 bug——重复应用同一个完整值是幂等的）。
    #[test]
    fn idempotent_form_catches_exactly_the_three_real_bugs() {
        let ops = gen_ops(2000, 19);
        for bug in [Bug::MergeReversed, Bug::TombstoneIgnored, Bug::DropMessage] {
            let r = run_idem(&ops, bug);
            assert_ne!(r.rt, r.ck, "幂等形态没抓到 {bug:?}");
        }
        let r = run_idem(&ops, Bug::DoubleApply);
        assert_eq!(r.rt, r.ck, "幂等形态对 DoubleApply 报警了，而它在这个形态下不是 bug");
        assert_eq!(r.ck, r.truth, "幂等形态在 DoubleApply 下账应当仍然是对的");
    }

    /// 操作流必须同时含 Set 与 Del，否则墓碑那一类注入根本走不到。
    #[test]
    fn generated_ops_contain_both_kinds() {
        let ops = gen_ops(2000, 23);
        assert!(ops.iter().any(|o| matches!(o, Op::Set(..))), "没有 Set");
        assert!(ops.iter().any(|o| matches!(o, Op::Del(..))), "没有 Del —— 墓碑注入走不到");
    }
}
