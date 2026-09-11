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
struct Transaction { writes: Vec<(u32, u64)> }

/// 一条 journal 记录。`transaction_number` 与 `commit` 只有在带边界字段的臂里才被重放看
#[derive(Debug, Clone)]
struct Record { writes: Vec<(u32, u64)>, transaction_number: u32, commit: bool }

#[derive(Debug, Clone, Copy, PartialEq)]
enum Arm {
    /// 一事务恰一条记录
    OneToOne,
    /// 一事务跨多条记录，记录头**没有**边界字段
    SplitNoBoundary,
    /// 一事务跨多条记录，记录头**有**事务号 + 提交标记
    SplitWithBoundary,
}

/// 造一批事务：第 t 个事务写 `writes_per_transaction` 条，key 与 value 都由 t 唯一决定
/// ⇒ 任何「半个事务」的状态都不可能与某个已提交状态相等，除非它真的落在边界上。
fn make_transactions(transaction_count: u32, writes_per_transaction: u32) -> Vec<Transaction> {
    (0..transaction_count).map(|transaction_index| Transaction {
        writes: (0..writes_per_transaction)
            .map(|write_index| (transaction_index * writes_per_transaction + write_index, (transaction_index as u64 + 1) * 1_000_003 + write_index as u64))
            .collect(),
    }).collect()
}

fn encode(transactions: &[Transaction], arm: Arm, chunk: u32) -> Vec<Record> {
    let mut encoded_records = Vec::new();
    for (transaction_index, transaction) in transactions.iter().enumerate() {
        match arm {
            Arm::OneToOne => encoded_records.push(Record { writes: transaction.writes.clone(), transaction_number: transaction_index as u32, commit: true }),
            _ => {
                let write_chunks: Vec<&[(u32, u64)]> = transaction.writes.chunks(chunk.max(1) as usize).collect();
                let last_chunk_index = write_chunks.len() - 1;
                for (chunk_index, chunk_writes) in write_chunks.iter().enumerate() {
                    encoded_records.push(Record { writes: chunk_writes.to_vec(), transaction_number: transaction_index as u32, commit: chunk_index == last_chunk_index });
                }
            }
        }
    }
    encoded_records
}

/// 重放一个前缀。带边界字段时，丢掉「提交标记还没出现」的那个事务的全部记录。
fn replay(encoded_records: &[Record], prefix_record_count: usize, honor_boundary: bool) -> BTreeMap<u32, u64> {
    let prefix = &encoded_records[..prefix_record_count];
    let last_committed_transaction_number: Option<u32> = if honor_boundary {
        prefix.iter().rev().find(|record| record.commit).map(|record| record.transaction_number)
    } else { None };
    let mut replayed_state = BTreeMap::new();
    for record in prefix {
        if honor_boundary {
            match last_committed_transaction_number { Some(committed_transaction_number) if record.transaction_number > committed_transaction_number => continue, None => continue, _ => {} }
        }
        for &(key, value) in &record.writes { replayed_state.insert(key, value); }
    }
    replayed_state
}

/// 真值：恰好前 n 个事务提交之后的状态，n = 0..=事务数。
fn truth_states(transactions: &[Transaction]) -> Vec<BTreeMap<u32, u64>> {
    let mut states_after_each_commit = vec![BTreeMap::new()];
    let mut cumulative_state = BTreeMap::new();
    for transaction in transactions {
        for &(key, value) in &transaction.writes { cumulative_state.insert(key, value); }
        states_after_each_commit.push(cumulative_state.clone());
    }
    states_after_each_commit
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ArmMeasurement {
    records: u64,
    /// 可截断的前缀总数 = 记录条数 + 1
    prefixes: u64,
    /// 落在事务**中间**的前缀数（由构造直接算出）
    cut_prefixes: u64,
    /// 重放后不等于任何一个真值状态的前缀数
    illegal: u64,
}

fn measure(transactions: &[Transaction], arm: Arm, chunk: u32) -> ArmMeasurement {
    let encoded_records = encode(transactions, arm, chunk);
    let truth = truth_states(transactions);
    let honor_boundary = matches!(arm, Arm::SplitWithBoundary);
    let mut illegal = 0u64;
    let mut cut_prefix_count = 0u64;
    // ⚠️ `prefixes` 数的是**真的枚举过的**那些，不是 encoded_records.len()+1 这个应然值。
    // 变异测试补出来的：把循环上界从 0..=len 改成 0..len，八个测试一个都没红——
    // 因为那条绝对值断言比的是两个都不来自循环的数（`rules/test-discipline.md`）。
    let mut enumerated = 0u64;
    for prefix_record_count in 0..=encoded_records.len() {
        enumerated += 1;
        // 「落在事务中间」= 前缀的最后一条记录不是该事务的提交记录
        if prefix_record_count > 0 && !encoded_records[prefix_record_count - 1].commit { cut_prefix_count += 1; }
        let replayed_state = replay(&encoded_records, prefix_record_count, honor_boundary);
        if !truth.iter().any(|truth_state| *truth_state == replayed_state) { illegal += 1; }
    }
    ArmMeasurement { records: encoded_records.len() as u64, prefixes: enumerated, cut_prefixes: cut_prefix_count, illegal }
}

fn main() {
    let mut emitter = Emitter::new();
    let transaction_count = 200u32;
    let writes_per_transaction = 8u32;      // D25 已定的粗粒度：一次 fsync 带 8 叶
    println!("{}", emitter.emit_raw(&format!(
        "name=config txns={transaction_count} writes_per_txn={writes_per_transaction}")));
    let transactions = make_transactions(transaction_count, writes_per_transaction);
    for chunk in [1u32, 2, 4] {
        for arm in [Arm::OneToOne, Arm::SplitNoBoundary, Arm::SplitWithBoundary] {
            let measurement = measure(&transactions, arm, chunk);
            println!("{}", emitter.emit_raw(&format!(
                "name=cell chunk={chunk} arm={arm:?} records={} prefixes={} \
                 cut_prefixes={} illegal={}",
                measurement.records, measurement.prefixes, measurement.cut_prefixes, measurement.illegal)));
        }
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_hundred_transactions_of_eight_writes() -> Vec<Transaction> { make_transactions(200, 8) }

    /// **绝对值断言 1**：**真的枚举过的**前缀数恰等于记录条数 + 1。
    /// ⚠️ 比的一侧来自循环、另一侧来自算术——两侧都取应然值的话，
    /// 少枚举一个前缀不会被任何测试看见（变异 M4 实测）。
    #[test]
    fn prefix_count_is_exactly_records_plus_one() {
        for chunk in [1u32, 2, 4] {
            for arm in [Arm::OneToOne, Arm::SplitNoBoundary, Arm::SplitWithBoundary] {
                let measurement = measure(&two_hundred_transactions_of_eight_writes(), arm, chunk);
                assert_eq!(measurement.prefixes, measurement.records + 1, "前缀数该是记录数 + 1（{arm:?} chunk={chunk}）");
            }
        }
    }

    /// **绝对值断言 2**：记录条数由构造直接算出。
    /// 一事务一条 ⇒ 200 条；按 chunk 切 ⇒ 200 × ceil(8/chunk)。
    #[test]
    fn record_count_matches_independently_computed_arithmetic() {
        assert_eq!(measure(&two_hundred_transactions_of_eight_writes(), Arm::OneToOne, 1).records, 200);
        assert_eq!(measure(&two_hundred_transactions_of_eight_writes(), Arm::SplitNoBoundary, 1).records, 200 * 8);
        assert_eq!(measure(&two_hundred_transactions_of_eight_writes(), Arm::SplitNoBoundary, 2).records, 200 * 4);
        assert_eq!(measure(&two_hundred_transactions_of_eight_writes(), Arm::SplitNoBoundary, 4).records, 200 * 2);
    }

    /// **绝对值断言 3**：落在事务中间的前缀数由构造直接算出
    /// = 事务数 × (每事务记录数 − 1)。
    #[test]
    fn cut_prefix_count_matches_independently_computed_arithmetic() {
        assert_eq!(measure(&two_hundred_transactions_of_eight_writes(), Arm::OneToOne, 1).cut_prefixes, 0, "一事务一条，切不开");
        assert_eq!(measure(&two_hundred_transactions_of_eight_writes(), Arm::SplitNoBoundary, 1).cut_prefixes, 200 * (8 - 1));
        assert_eq!(measure(&two_hundred_transactions_of_eight_writes(), Arm::SplitNoBoundary, 2).cut_prefixes, 200 * (4 - 1));
        assert_eq!(measure(&two_hundred_transactions_of_eight_writes(), Arm::SplitNoBoundary, 4).cut_prefixes, 200 * (2 - 1));
    }

    /// **判据 1**：一事务一条记录 ⇒ 任意前缀都合法，违反数恒为 0。
    #[test]
    fn one_to_one_never_produces_an_illegal_prefix() {
        for chunk in [1u32, 2, 4] {
            assert_eq!(measure(&two_hundred_transactions_of_eight_writes(), Arm::OneToOne, chunk).illegal, 0,
                "一事务一条记录时任意前缀都该合法（chunk={chunk} 对本臂无效）");
        }
    }

    /// **判据 2 / 阳性对照**：跨多条且没有边界字段 ⇒ 违反数必须显著非零，
    /// 且**恰等于落在事务中间的前缀数**——因为构造保证半个事务的状态与任何已提交状态都不等。
    #[test]
    fn splitting_without_a_boundary_field_breaks_exactly_the_cut_prefixes() {
        for chunk in [1u32, 2, 4] {
            let measurement = measure(&two_hundred_transactions_of_eight_writes(), Arm::SplitNoBoundary, chunk);
            assert!(measurement.illegal > 0, "跨多条记录该造出非法前缀（chunk={chunk}）");
            assert_eq!(measurement.illegal, measurement.cut_prefixes,
                "非法前缀该恰是被切开的那些（chunk={chunk}）");
        }
    }

    /// **判据 3**：加上事务号 + 提交标记之后违反数回到 0。
    #[test]
    fn adding_a_boundary_field_restores_prefix_legality() {
        for chunk in [1u32, 2, 4] {
            let measurement = measure(&two_hundred_transactions_of_eight_writes(), Arm::SplitWithBoundary, chunk);
            assert_eq!(measurement.illegal, 0, "带边界字段时任意前缀该重新合法（chunk={chunk}）");
            assert!(measurement.cut_prefixes > 0, "它仍然是跨多条记录的——切开的前缀还在（chunk={chunk}）");
        }
    }

    /// **合法性是按状态判的，不是按边界判的。**
    /// 直接验重放函数：落在事务中间的那个前缀，重放出来的 map 与前后两个真值都不等。
    #[test]
    fn legality_is_decided_by_comparing_states_not_boundaries() {
        let transactions = make_transactions(3, 4);
        let encoded_records = encode(&transactions, Arm::SplitNoBoundary, 1);
        let truth = truth_states(&transactions);
        let mid_transaction_state = replay(&encoded_records, 6, false);      // 第 2 个事务写到一半
        assert_ne!(mid_transaction_state, truth[1]);
        assert_ne!(mid_transaction_state, truth[2]);
        assert_eq!(mid_transaction_state.len(), 6, "重放出 6 条写入：第 1 个事务 4 条 + 第 2 个事务的前 2 条");
    }

    /// **边界字段真的在丢弃未提交那一段**，不是靠别的机制蒙对。
    #[test]
    fn the_boundary_field_discards_the_uncommitted_tail() {
        let transactions = make_transactions(3, 4);
        let encoded_records = encode(&transactions, Arm::SplitWithBoundary, 1);
        let honored = replay(&encoded_records, 6, true);
        assert_eq!(honored, truth_states(&transactions)[1], "该退回到第 1 个事务提交后的状态");
        assert_eq!(honored.len(), 4);
    }
}
