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

const KEY_COUNT: u64 = 512;
const BATCH: usize = 64;

#[derive(Clone, Copy)]
enum Operation { Set(u64, i64), Delete(u64) }

fn generate_operations(operation_count: usize, seed: u64) -> Vec<Operation> {
    let mut xorshift_state = seed | 1;
    let mut next_random = || { xorshift_state ^= xorshift_state >> 12; xorshift_state ^= xorshift_state << 25; xorshift_state ^= xorshift_state >> 27; xorshift_state.wrapping_mul(0x2545F4914F6CDD1D) };
    (0..operation_count).map(|_| {
        let key = next_random() % KEY_COUNT;
        if next_random() % 4 == 0 { Operation::Delete(key) } else { Operation::Set(key, (next_random() % 1000 + 1) as i64) }
    }).collect()
}

/// 真值：按顺序放进一张表求和。没有合并规则可以写错。
fn ground_truth(operations: &[Operation]) -> i64 {
    let mut live_sizes: BTreeMap<u64, i64> = BTreeMap::new();
    for operation in operations {
        match *operation { Operation::Set(key, size) => { live_sizes.insert(key, size); } Operation::Delete(key) => { live_sizes.remove(&key); } }
    }
    live_sizes.values().sum()
}

/// 一轮的三个读数。
struct Reading { runtime_total: i64, checker_total: i64, truth: i64, entries: usize }

// ───────── 形态 A：幂等完整值 (key, Some(size)) / (key, None)=tombstone ─────────
// 权威是内容。账是缓存。checker 遍历内容求和——**独立实现，不碰下面的合并函数**。

fn flush_idempotent(buffered_entries: &[(u64, Option<i64>)], tree: &mut BTreeMap<u64, i64>, bug: Bug) {
    // 合并（后者胜）后落树。bug 注在这里 = 注在 flush 侧。
    let mut merged: BTreeMap<u64, Option<i64>> = BTreeMap::new();
    for &(key, size_or_tombstone) in buffered_entries {
        match bug {
            Bug::MergeReversed => { merged.entry(key).or_insert(size_or_tombstone); }        // 取先者胜
            Bug::TombstoneIgnored => { if size_or_tombstone.is_some() { merged.insert(key, size_or_tombstone); } } // 墓碑当普通值
            _ => { merged.insert(key, size_or_tombstone); }
        }
    }
    for (&key, &size_or_tombstone) in &merged {
        match size_or_tombstone { Some(size) => { tree.insert(key, size); } None => { tree.remove(&key); } }
    }
}

fn run_idempotent(operations: &[Operation], bug: Bug) -> Reading {
    let mut tree: BTreeMap<u64, i64> = BTreeMap::new();
    let mut shadow: BTreeMap<u64, i64> = BTreeMap::new(); // 运行时对「当前值」的认识
    let mut runtime_total: i64 = 0;
    let mut entries = 0usize;
    for chunk in operations.chunks(BATCH) {
        let mut buffered_entries: Vec<(u64, Option<i64>)> = Vec::with_capacity(chunk.len());
        for operation in chunk {
            // 入缓冲的同时增量记账——这一步与 flush 无关，是事务提交侧的代码
            let (key, new_size_or_tombstone) = match *operation { Operation::Set(key, size) => (key, Some(size)), Operation::Delete(key) => (key, None) };
            let old_size = shadow.get(&key).copied().unwrap_or(0);
            let new_size = new_size_or_tombstone.unwrap_or(0);
            runtime_total += new_size - old_size;
            match new_size_or_tombstone { Some(size) => { shadow.insert(key, size); } None => { shadow.remove(&key); } }
            buffered_entries.push((key, new_size_or_tombstone));
        }
        if bug == Bug::DropMessage && !buffered_entries.is_empty() { buffered_entries.pop(); }
        if bug == Bug::DoubleApply { let duplicated_entries = buffered_entries.clone(); buffered_entries.extend(duplicated_entries); }
        entries += buffered_entries.len();
        flush_idempotent(&buffered_entries, &mut tree, bug);
    }
    Reading { runtime_total, checker_total: tree.values().sum(), truth: ground_truth(operations), entries }
}

// ───────── 形态 B：增量 Δ (key, Δbytes)。合并 = 累加。 ─────────

/// 运行时与 checker **共用的同一个累加函数**。bug 注在这里，于是两侧一起错。
fn accumulate(accumulator: &mut i64, delta: i64, bug: Bug) {
    match bug { Bug::MergeReversed => *accumulator -= delta, _ => *accumulator += delta }
}

fn run_delta(operations: &[Operation], bug: Bug) -> Reading {
    let mut shadow: BTreeMap<u64, i64> = BTreeMap::new();
    let mut runtime_total: i64 = 0;
    let mut delta_log: Vec<(u64, i64)> = Vec::new(); // 持久化下来的那份 Δ 日志
    for chunk in operations.chunks(BATCH) {
        let mut buffered_deltas: Vec<(u64, i64)> = Vec::with_capacity(chunk.len());
        for operation in chunk {
            let (key, delta) = match *operation {
                Operation::Set(key, new_size) => { let old_size = shadow.get(&key).copied().unwrap_or(0);
                                   shadow.insert(key, new_size); (key, new_size - old_size) }
                Operation::Delete(key) => { let old_size = shadow.get(&key).copied().unwrap_or(0);
                                if bug == Bug::TombstoneIgnored { (key, 0) }   // 删除不产生负 Δ
                                else { shadow.remove(&key); (key, -old_size) } }
            };
            buffered_deltas.push((key, delta));
        }
        if bug == Bug::DropMessage && !buffered_deltas.is_empty() { buffered_deltas.pop(); }
        if bug == Bug::DoubleApply { let duplicated_deltas = buffered_deltas.clone(); buffered_deltas.extend(duplicated_deltas); }
        for &(_, delta) in &buffered_deltas { accumulate(&mut runtime_total, delta, bug); }
        delta_log.extend(buffered_deltas);
    }
    // checker：重放同一份日志，用同一个 accumulate
    let mut checker_total = 0i64;
    for &(_, delta) in &delta_log { accumulate(&mut checker_total, delta, bug); }
    Reading { runtime_total, checker_total, truth: ground_truth(operations), entries: delta_log.len() }
}

fn main() {
    let mut emitter = Emitter::new();
    let seeds: [u64; 5] = [11, 22, 33, 44, 55];
    let operation_count = 20_000usize;
    println!("{}", emitter.emit_raw(&format!(
        "name=config keys={KEY_COUNT} ops={operation_count} batch={BATCH} seeds={}", seeds.len())));
    for form in [Form::Idempotent, Form::Delta] {
        let form_name = if form == Form::Idempotent { "idempotent" } else { "delta" };
        for bug in [Bug::None, Bug::MergeReversed, Bug::TombstoneIgnored, Bug::DropMessage, Bug::DoubleApply] {
            let (mut invariant_fired_count, mut value_correct_count, mut flushed_entry_count) = (0u32, 0u32, 0usize);
            for seed in seeds {
                let operations = generate_operations(operation_count, seed);
                let reading = if form == Form::Idempotent { run_idempotent(&operations, bug) } else { run_delta(&operations, bug) };
                if reading.runtime_total != reading.checker_total { invariant_fired_count += 1; }
                if reading.runtime_total == reading.truth { value_correct_count += 1; }
                flushed_entry_count = reading.entries; // 各 seed 相同：条目数只由 operation_count/BATCH 与注入决定
            }
            println!("{}", emitter.emit_raw(&format!(
                "name=case form={form_name} bug={bug:?} i31_red={invariant_fired_count}/{seed_count} value_correct={value_correct_count}/{seed_count} flushed_entries={flushed_entry_count}",
                seed_count = seeds.len())));
        }
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **真值必须是独立算出来的**，不许走被测的那两条合并路径。
    /// 它是本实验唯一的裁判——它错了，四类注入的判定全部作废。
    #[test]
    fn ground_truth_is_a_plain_replay() {
        let operations = vec![Operation::Set(1, 10), Operation::Set(2, 20), Operation::Set(1, 5), Operation::Delete(2)];
        assert_eq!(ground_truth(&operations), 5, "真值应当只剩 key1=5");
        assert_eq!(ground_truth(&[]), 0);
        // 删一个不存在的 key 不该改变结果
        assert_eq!(ground_truth(&[Operation::Set(1, 7), Operation::Delete(9)]), 7);
    }

    /// **不注入 bug 时两种形态都必须算对账**，否则「注入后算错」就说明不了任何事。
    #[test]
    fn both_forms_are_correct_without_any_bug() {
        let operations = generate_operations(2000, 11);
        for (name, reading) in [("幂等", run_idempotent(&operations, Bug::None)), ("增量", run_delta(&operations, Bug::None))] {
            assert_eq!(reading.runtime_total, reading.truth, "{name}形态在无 bug 时运行时账就不对");
            assert_eq!(reading.checker_total, reading.truth, "{name}形态在无 bug 时 checker 账就不对");
        }
    }

    /// **四类注入必须真的把账算错**——否则「I-3.1 抓不到」是因为根本没有错可抓，
    /// 那样得出的「判别力为零」是伪的。这条是本实验非平凡性的根。
    #[test]
    fn every_injected_bug_actually_corrupts_the_accounting() {
        let operations = generate_operations(2000, 13);
        for bug in [Bug::MergeReversed, Bug::TombstoneIgnored, Bug::DropMessage, Bug::DoubleApply] {
            let delta_reading = run_delta(&operations, bug);
            assert_ne!(delta_reading.runtime_total, delta_reading.truth, "增量形态注入 {bug:?} 之后账居然还是对的");
        }
    }

    /// **I-3.1 的判别力就是 `runtime_total == checker_total` 这一个比较**：
    /// 增量形态下两者同源 ⇒ 永远相等 ⇒ 判别力为零。
    /// 这条把「为什么抓不到」钉死在机制上，不是钉在某一次的数字上。
    #[test]
    fn delta_form_makes_the_invariant_blind_by_construction() {
        let operations = generate_operations(2000, 17);
        for bug in [Bug::None, Bug::MergeReversed, Bug::TombstoneIgnored, Bug::DropMessage, Bug::DoubleApply] {
            let delta_reading = run_delta(&operations, bug);
            assert_eq!(delta_reading.runtime_total, delta_reading.checker_total, "增量形态下 I-3.1 竟然分开了 {bug:?}——那它就不是同源的了");
        }
    }

    /// 幂等形态下 I-3.1 必须抓到三类真 bug，且对 `DoubleApply` 不报警
    /// （那一类在幂等形态下根本不是 bug——重复应用同一个完整值是幂等的）。
    #[test]
    fn idempotent_form_catches_exactly_the_three_real_bugs() {
        let operations = generate_operations(2000, 19);
        for bug in [Bug::MergeReversed, Bug::TombstoneIgnored, Bug::DropMessage] {
            let reading = run_idempotent(&operations, bug);
            assert_ne!(reading.runtime_total, reading.checker_total, "幂等形态没抓到 {bug:?}");
        }
        let reading = run_idempotent(&operations, Bug::DoubleApply);
        assert_eq!(reading.runtime_total, reading.checker_total, "幂等形态对 DoubleApply 报警了，而它在这个形态下不是 bug");
        assert_eq!(reading.checker_total, reading.truth, "幂等形态在 DoubleApply 下账应当仍然是对的");
    }

    /// 操作流必须同时含 Set 与 Del，否则墓碑那一类注入根本走不到。
    #[test]
    fn generated_operations_contain_both_kinds() {
        let operations = generate_operations(2000, 23);
        assert!(operations.iter().any(|operation| matches!(operation, Operation::Set(..))), "没有 Set");
        assert!(operations.iter().any(|operation| matches!(operation, Operation::Delete(..))), "没有 Del —— 墓碑注入走不到");
    }
}
