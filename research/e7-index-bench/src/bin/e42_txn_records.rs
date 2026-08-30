//! E42：一个事务恰不恰好产生一条 journal 记录 —— D23 已定项 7。
//!
//! ## 它不是「选一个更好的」
//!
//! 仓里两处措辞给出两种读法，而两种都成立：
//! I-8.1 写「环大小 ≥ F × 任一事务的最坏 journal **占用**」——**占用是空间量纲，不是「一条」**；
//! 而 D23 死锁 3 说「算不下的操作必须能拆成若干个各自合法的事务」，读起来又像「一事务一记录」被维持着。
//! ⇒ **它从没被定过。**
//!
//! ## 判据（experiments.md E42）
//!
//! I-4.3（提交原子）与 D16「根 + journal **任意前缀**，replay 后合法」要求的是：
//! **任一前缀重放出来的状态，必须等于某个整数个已提交事务的状态。**
//! 本实验把「任意前缀」真的枚举一遍，看三条臂各自违反几次。
//!
//! - 臂 A（一事务恰一条记录）：违反数必须**恒为 0**。
//! - 臂 B（一事务跨多条记录、记录头没有边界字段）：必须**显著非零**——这就是「跨多条会出事」的量。
//! - 臂 B + 边界字段（事务号 + 提交标记）：必须**回到 0**。
//!
//! ⚠️ 合法性是**按状态判的，不是按边界判的**：重放出来的 map 要与某个真值 map 逐条相等。
//! 只查「前缀是不是落在边界上」等于把结论当判据用。

use e7_index_bench::Emitter;
use std::collections::BTreeMap;

/// 一个事务：若干条「幂等完整值」写入（D8 已定项 1 已定的形态）。
#[derive(Debug, Clone)]
struct Txn { writes: Vec<(u32, u64)> }

/// 一条 journal 记录。`txn` 与 `commit` 只有在带边界字段的臂里才被重放看
#[derive(Debug, Clone)]
struct Record { writes: Vec<(u32, u64)>, txn: u32, commit: bool }

#[derive(Debug, Clone, Copy, PartialEq)]
enum Arm {
    /// 一事务恰一条记录
    OneToOne,
    /// 一事务跨多条记录，记录头**没有**边界字段
    SplitNoBoundary,
    /// 一事务跨多条记录，记录头**有**事务号 + 提交标记
    SplitWithBoundary,
}

/// 造一批事务：第 t 个事务写 `writes_per_txn` 条，key 与 value 都由 t 唯一决定
/// ⇒ 任何「半个事务」的状态都不可能与某个已提交状态相等，除非它真的落在边界上。
fn make_txns(n: u32, writes_per_txn: u32) -> Vec<Txn> {
    (0..n).map(|t| Txn {
        writes: (0..writes_per_txn)
            .map(|w| (t * writes_per_txn + w, (t as u64 + 1) * 1_000_003 + w as u64))
            .collect(),
    }).collect()
}

fn encode(txns: &[Txn], arm: Arm, chunk: u32) -> Vec<Record> {
    let mut out = Vec::new();
    for (t, tx) in txns.iter().enumerate() {
        match arm {
            Arm::OneToOne => out.push(Record { writes: tx.writes.clone(), txn: t as u32, commit: true }),
            _ => {
                let parts: Vec<&[(u32, u64)]> = tx.writes.chunks(chunk.max(1) as usize).collect();
                let last = parts.len() - 1;
                for (i, p) in parts.iter().enumerate() {
                    out.push(Record { writes: p.to_vec(), txn: t as u32, commit: i == last });
                }
            }
        }
    }
    out
}

/// 重放一个前缀。带边界字段时，丢掉「提交标记还没出现」的那个事务的全部记录。
fn replay(recs: &[Record], upto: usize, honor_boundary: bool) -> BTreeMap<u32, u64> {
    let prefix = &recs[..upto];
    let committed: Option<u32> = if honor_boundary {
        prefix.iter().rev().find(|r| r.commit).map(|r| r.txn)
    } else { None };
    let mut m = BTreeMap::new();
    for r in prefix {
        if honor_boundary {
            match committed { Some(c) if r.txn > c => continue, None => continue, _ => {} }
        }
        for &(k, v) in &r.writes { m.insert(k, v); }
    }
    m
}

/// 真值：恰好前 n 个事务提交之后的状态，n = 0..=事务数。
fn truth_states(txns: &[Txn]) -> Vec<BTreeMap<u32, u64>> {
    let mut out = vec![BTreeMap::new()];
    let mut m = BTreeMap::new();
    for tx in txns {
        for &(k, v) in &tx.writes { m.insert(k, v); }
        out.push(m.clone());
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Out {
    records: u64,
    /// 可截断的前缀总数 = 记录条数 + 1
    prefixes: u64,
    /// 落在事务**中间**的前缀数（由构造直接算出）
    cut_prefixes: u64,
    /// 重放后不等于任何一个真值状态的前缀数
    illegal: u64,
}

fn measure(txns: &[Txn], arm: Arm, chunk: u32) -> Out {
    let recs = encode(txns, arm, chunk);
    let truth = truth_states(txns);
    let honor = matches!(arm, Arm::SplitWithBoundary);
    let mut illegal = 0u64;
    let mut cut = 0u64;
    // ⚠️ `prefixes` 数的是**真的枚举过的**那些，不是 recs.len()+1 这个应然值。
    // 变异测试补出来的：把循环上界从 0..=len 改成 0..len，八个测试一个都没红——
    // 因为那条绝对值断言比的是两个都不来自循环的数（`rules/test-discipline.md`）。
    let mut enumerated = 0u64;
    for upto in 0..=recs.len() {
        enumerated += 1;
        // 「落在事务中间」= 前缀的最后一条记录不是该事务的提交记录
        if upto > 0 && !recs[upto - 1].commit { cut += 1; }
        let st = replay(&recs, upto, honor);
        if !truth.iter().any(|t| *t == st) { illegal += 1; }
    }
    Out { records: recs.len() as u64, prefixes: enumerated, cut_prefixes: cut, illegal }
}

fn main() {
    let mut em = Emitter::new();
    let n_txns = 200u32;
    let writes_per_txn = 8u32;      // D25 已定的粗粒度：一次 fsync 带 8 叶
    println!("{}", em.emit_raw(&format!(
        "name=config txns={n_txns} writes_per_txn={writes_per_txn}")));
    let txns = make_txns(n_txns, writes_per_txn);
    for chunk in [1u32, 2, 4] {
        for arm in [Arm::OneToOne, Arm::SplitNoBoundary, Arm::SplitWithBoundary] {
            let o = measure(&txns, arm, chunk);
            println!("{}", em.emit_raw(&format!(
                "name=cell chunk={chunk} arm={arm:?} records={} prefixes={} \
                 cut_prefixes={} illegal={}",
                o.records, o.prefixes, o.cut_prefixes, o.illegal)));
        }
    }
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t() -> Vec<Txn> { make_txns(200, 8) }

    /// **绝对值断言 1**：**真的枚举过的**前缀数恰等于记录条数 + 1。
    /// ⚠️ 比的一侧来自循环、另一侧来自算术——两侧都取应然值的话，
    /// 少枚举一个前缀不会被任何测试看见（变异 M4 实测）。
    #[test]
    fn prefix_count_is_exactly_records_plus_one() {
        for chunk in [1u32, 2, 4] {
            for arm in [Arm::OneToOne, Arm::SplitNoBoundary, Arm::SplitWithBoundary] {
                let o = measure(&t(), arm, chunk);
                assert_eq!(o.prefixes, o.records + 1, "前缀数该是记录数 + 1（{arm:?} chunk={chunk}）");
            }
        }
    }

    /// **绝对值断言 2**：记录条数由构造直接算出。
    /// 一事务一条 ⇒ 200 条；按 chunk 切 ⇒ 200 × ceil(8/chunk)。
    #[test]
    fn record_count_matches_independently_computed_arithmetic() {
        assert_eq!(measure(&t(), Arm::OneToOne, 1).records, 200);
        assert_eq!(measure(&t(), Arm::SplitNoBoundary, 1).records, 200 * 8);
        assert_eq!(measure(&t(), Arm::SplitNoBoundary, 2).records, 200 * 4);
        assert_eq!(measure(&t(), Arm::SplitNoBoundary, 4).records, 200 * 2);
    }

    /// **绝对值断言 3**：落在事务中间的前缀数由构造直接算出
    /// = 事务数 × (每事务记录数 − 1)。
    #[test]
    fn cut_prefix_count_matches_independently_computed_arithmetic() {
        assert_eq!(measure(&t(), Arm::OneToOne, 1).cut_prefixes, 0, "一事务一条，切不开");
        assert_eq!(measure(&t(), Arm::SplitNoBoundary, 1).cut_prefixes, 200 * (8 - 1));
        assert_eq!(measure(&t(), Arm::SplitNoBoundary, 2).cut_prefixes, 200 * (4 - 1));
        assert_eq!(measure(&t(), Arm::SplitNoBoundary, 4).cut_prefixes, 200 * (2 - 1));
    }

    /// **判据 1**：一事务一条记录 ⇒ 任意前缀都合法，违反数恒为 0。
    #[test]
    fn one_to_one_never_produces_an_illegal_prefix() {
        for chunk in [1u32, 2, 4] {
            assert_eq!(measure(&t(), Arm::OneToOne, chunk).illegal, 0,
                "一事务一条记录时任意前缀都该合法（chunk={chunk} 对本臂无效）");
        }
    }

    /// **判据 2 / 阳性对照**：跨多条且没有边界字段 ⇒ 违反数必须显著非零，
    /// 且**恰等于落在事务中间的前缀数**——因为构造保证半个事务的状态与任何已提交状态都不等。
    #[test]
    fn splitting_without_a_boundary_field_breaks_exactly_the_cut_prefixes() {
        for chunk in [1u32, 2, 4] {
            let o = measure(&t(), Arm::SplitNoBoundary, chunk);
            assert!(o.illegal > 0, "跨多条记录该造出非法前缀（chunk={chunk}）");
            assert_eq!(o.illegal, o.cut_prefixes,
                "非法前缀该恰是被切开的那些（chunk={chunk}）");
        }
    }

    /// **判据 3**：加上事务号 + 提交标记之后违反数回到 0。
    #[test]
    fn a_boundary_field_restores_prefix_legality() {
        for chunk in [1u32, 2, 4] {
            let o = measure(&t(), Arm::SplitWithBoundary, chunk);
            assert_eq!(o.illegal, 0, "带边界字段时任意前缀该重新合法（chunk={chunk}）");
            assert!(o.cut_prefixes > 0, "它仍然是跨多条记录的——切开的前缀还在（chunk={chunk}）");
        }
    }

    /// **合法性是按状态判的，不是按边界判的。**
    /// 直接验重放函数：落在事务中间的那个前缀，重放出来的 map 与前后两个真值都不等。
    #[test]
    fn legality_is_decided_by_comparing_states_not_boundaries() {
        let txns = make_txns(3, 4);
        let recs = encode(&txns, Arm::SplitNoBoundary, 1);
        let truth = truth_states(&txns);
        let mid = replay(&recs, 6, false);      // 第 2 个事务写到一半
        assert_ne!(mid, truth[1]);
        assert_ne!(mid, truth[2]);
        assert_eq!(mid.len(), 6, "重放出 6 条写入：第 1 个事务 4 条 + 第 2 个事务的前 2 条");
    }

    /// **边界字段真的在丢弃未提交那一段**，不是靠别的机制蒙对。
    #[test]
    fn the_boundary_field_discards_the_uncommitted_tail() {
        let txns = make_txns(3, 4);
        let recs = encode(&txns, Arm::SplitWithBoundary, 1);
        let honored = replay(&recs, 6, true);
        assert_eq!(honored, truth_states(&txns)[1], "该退回到第 1 个事务提交后的状态");
        assert_eq!(honored.len(), 4);
    }
}
