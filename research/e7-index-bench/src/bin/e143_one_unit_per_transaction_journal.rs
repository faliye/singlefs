//! E143：一事务一单元下的 journal 账——C310（事务切分纪律与记录数口径打架）。
//!
//! 跑前登记 `research/prompts/e143-preregistration.md`。用户 2026-09-13 定案：切分纪律是一事务一单元（D16（发布语义） 已定项 5 末段照旧）。
//! 三条臂：甲 = 今天条款（一条记录一个事务）、乙 = 一条记录装多个事务（对照，点名项各带事务号）、丙 = 一次 fsync 一个事务（E75 的旧读法，对照）。
//! 六个负载点照 D25（目标负载优先级）；T_dirty 满窗按两种口径；丙按 E91 的口径复现 1847 条 / 7388 KiB 当跨装置闸。纯算术。

use e7_index_bench::Emitter;

/// D23（journal 的角色与格式） 已定项 12。
const JOURNAL_RECORD_BYTES: u64 = 4096;
/// 记录头 78 + 三笔已定增量 = 95（字节表六）。
const JOURNAL_HEADER_BYTES_TODAY: u64 = 95;
/// D23（journal 的角色与格式） 已定项 4 口径的点名项。
const ITEM_BYTES_TODAY: u64 = 56;
/// 乙臂：点名项各带事务号 8。
const ITEM_BYTES_WITH_TRANSACTION: u64 = 64;
/// 乙臂：记录头去掉事务号 8 + 提交标记 1，加首末事务号 16。
const JOURNAL_HEADER_BYTES_MULTI: u64 = JOURNAL_HEADER_BYTES_TODAY - 9 + 16;
const DEVICES: u64 = 2;
/// D4（校验和位置） 已定项 5。
const DATA_UNIT_BYTES: u64 = 32768;
/// D8（核心索引结构） 已定项 2；E91 把 T_dirty 满窗按它算项数。
const NODE_BYTES: u64 = 16384;
/// D16（发布语义） 已定项 5。
const DIRTY_THRESHOLD_BYTES: u64 = 2 << 30;
/// 字节表零的预想环长。
const RING_BYTES: u64 = 64 << 20;
/// I-8.1（环几何够大） 的安全系数，超级块预想 3。
const SAFETY_FACTOR: u64 = 3;
/// E44（序号位宽的本机实测与代价）。
const FSYNC_PER_SECOND: u64 = 2785;

#[derive(Clone, Copy, Debug)]
struct Load {
    name: &'static str,
    batch: u64,
    leaves: u64,
    ancestors: u64,
}

/// D25（目标负载优先级） 的六个点；metaheavy 的 5.937 个祖先按项数向上取整成 6。
const LOADS: [Load; 6] = [
    Load { name: "rand", batch: 1, leaves: 1, ancestors: 4 },
    Load { name: "multistream", batch: 1, leaves: 1, ancestors: 4 },
    Load { name: "seq", batch: 1, leaves: 8, ancestors: 4 },
    Load { name: "metaheavy", batch: 1, leaves: 2, ancestors: 6 },
    Load { name: "seq", batch: 10, leaves: 80, ancestors: 4 },
    Load { name: "multistream", batch: 10, leaves: 10, ancestors: 31 },
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    /// 甲：每个数据单元一个事务、一条记录一个事务；祖先并进这次 fsync 的末条记录。
    OneRecordPerTransaction,
    /// 乙：每个数据单元一个事务，一条记录装多个事务，点名项各带事务号。
    ManyTransactionsPerRecord,
    /// 丙：一次 fsync 一个事务（E75 的旧读法），用户已否，只作对照。
    OneTransactionPerFsync,
}

const ARMS: [Arm; 3] = [Arm::OneRecordPerTransaction, Arm::ManyTransactionsPerRecord, Arm::OneTransactionPerFsync];

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::OneRecordPerTransaction => "jia",
            Arm::ManyTransactionsPerRecord => "yi",
            Arm::OneTransactionPerFsync => "bing",
        }
    }
    fn items_per_record(self) -> u64 {
        match self {
            Arm::OneRecordPerTransaction | Arm::OneTransactionPerFsync => items_fitting(JOURNAL_HEADER_BYTES_TODAY, ITEM_BYTES_TODAY),
            Arm::ManyTransactionsPerRecord => items_fitting(JOURNAL_HEADER_BYTES_MULTI, ITEM_BYTES_WITH_TRANSACTION),
        }
    }
    /// 一次 fsync 写几条记录。
    fn records_per_fsync(self, load: Load) -> u64 {
        match self {
            Arm::OneRecordPerTransaction => {
                // 每个数据单元一条；祖先并进末条，装不下的部分再开记录。
                let last_record_items = 1 + load.ancestors;
                load.leaves + last_record_items.div_ceil(self.items_per_record()) - 1
            }
            Arm::ManyTransactionsPerRecord | Arm::OneTransactionPerFsync => (load.leaves + load.ancestors).div_ceil(self.items_per_record()),
        }
    }
    /// 一次 fsync 的事务数（映射 key 的写序要它）。
    fn transactions_per_fsync(self, load: Load) -> u64 {
        match self {
            Arm::OneRecordPerTransaction | Arm::ManyTransactionsPerRecord => load.leaves,
            Arm::OneTransactionPerFsync => 1,
        }
    }
    /// T_dirty 满窗的记录数：甲、乙按「脏字节 = 数据单元字节」，丙按 E91 的「16 KiB 节点数」口径。
    fn records_in_dirty_window(self) -> u64 {
        match self {
            Arm::OneRecordPerTransaction => DIRTY_THRESHOLD_BYTES / DATA_UNIT_BYTES,
            Arm::ManyTransactionsPerRecord => (DIRTY_THRESHOLD_BYTES / DATA_UNIT_BYTES).div_ceil(self.items_per_record()),
            Arm::OneTransactionPerFsync => (DIRTY_THRESHOLD_BYTES / NODE_BYTES).div_ceil(self.items_per_record()),
        }
    }
}

/// 一条记录装几项：(记录 − 头) ÷ 项宽，向下取整。
fn items_fitting(header_bytes: u64, item_bytes: u64) -> u64 {
    (JOURNAL_RECORD_BYTES - header_bytes) / item_bytes
}

fn journal_bytes(records: u64) -> u64 {
    records * JOURNAL_RECORD_BYTES * DEVICES
}

fn data_bytes(load: Load) -> u64 {
    load.leaves * DATA_UNIT_BYTES * DEVICES
}

/// 一条记录的最坏占用（I-8.1 按「任一事务」读）：三条臂的一个事务都装进一条记录。
fn worst_transaction_bytes(arm: Arm, load: Load) -> u64 {
    match arm {
        Arm::OneRecordPerTransaction | Arm::ManyTransactionsPerRecord => JOURNAL_RECORD_BYTES,
        Arm::OneTransactionPerFsync => (load.leaves + load.ancestors).div_ceil(arm.items_per_record()) * JOURNAL_RECORD_BYTES,
    }
}

/// 64 MiB 环、F = 3 下甲臂能容的 T_dirty：环 ÷ F ÷ 记录 = 记录数 = 数据单元数，再乘单元字节。
fn dirty_bytes_the_ring_allows_for_jia() -> u64 {
    RING_BYTES / SAFETY_FACTOR / JOURNAL_RECORD_BYTES * DATA_UNIT_BYTES
}

/// 码 1 映射 key 的写序段 (实例代号, 事务号)：一次 fsync 的各数据单元拿到几个不同的事务号。
fn distinct_write_orders(arm: Arm, load: Load) -> u64 {
    let transactions = arm.transactions_per_fsync(load);
    (1..=load.leaves).map(|unit_index| ((unit_index - 1) % transactions) + 1).collect::<std::collections::BTreeSet<u64>>().len() as u64
}

fn emit(emitter: &mut Emitter, body: &str) {
    println!("{}", emitter.emit_raw(body));
}

fn main() {
    let mut emitter = Emitter::new();
    emit(&mut emitter, &format!("name=config record_bytes={JOURNAL_RECORD_BYTES} header_today={JOURNAL_HEADER_BYTES_TODAY} header_multi={JOURNAL_HEADER_BYTES_MULTI} item_today={ITEM_BYTES_TODAY} item_multi={ITEM_BYTES_WITH_TRANSACTION} devices={DEVICES} t_dirty_bytes={DIRTY_THRESHOLD_BYTES} ring_bytes={RING_BYTES} safety_factor={SAFETY_FACTOR} fsync_per_second={FSYNC_PER_SECOND}"));
    for arm in ARMS {
        emit(&mut emitter, &format!("name=capacity arm={} items_per_record={}", arm.name(), arm.items_per_record()));
    }
    for load in LOADS {
        for arm in ARMS {
            let records = arm.records_per_fsync(load);
            let journal = journal_bytes(records);
            let data = data_bytes(load);
            emit(&mut emitter, &format!(
                "name=fsync load={} batch={} leaves={} ancestors={} arm={} transactions={} records={records} journal_bytes_two_devices={journal} data_bytes_two_devices={data} journal_over_data_bp={} records_per_second={} journal_kib_per_second={}",
                load.name, load.batch, load.leaves, load.ancestors, arm.name(), arm.transactions_per_fsync(load), journal * 10_000 / data, records * FSYNC_PER_SECOND, journal * FSYNC_PER_SECOND / 1024
            ));
        }
    }
    for arm in ARMS {
        let records = arm.records_in_dirty_window();
        let bytes_kib = records * JOURNAL_RECORD_BYTES / 1024;
        emit(&mut emitter, &format!("name=window arm={} records={records} bytes_kib={bytes_kib} ring_kib={} window_over_ring_milli={}", arm.name(), RING_BYTES / 1024, bytes_kib * 1000 / (RING_BYTES / 1024)));
    }
    let sequential_batch_one = LOADS[2];
    for arm in ARMS {
        let worst = worst_transaction_bytes(arm, sequential_batch_one);
        let window_bytes = arm.records_in_dirty_window() * JOURNAL_RECORD_BYTES;
        emit(&mut emitter, &format!(
            "name=ring arm={} worst_transaction_bytes={worst} lower_bound_any_transaction_kib={} lower_bound_dirty_window_kib={}",
            arm.name(), SAFETY_FACTOR * worst / 1024, SAFETY_FACTOR * window_bytes / 1024
        ));
    }
    emit(&mut emitter, &format!("name=ring_cap arm=jia ring_kib={} safety_factor={SAFETY_FACTOR} dirty_bytes_allowed={} dirty_kib_allowed={}", RING_BYTES / 1024, dirty_bytes_the_ring_allows_for_jia(), dirty_bytes_the_ring_allows_for_jia() / 1024));
    for arm in ARMS {
        emit(&mut emitter, &format!("name=keys load=seq batch=1 arm={} data_units={} distinct_write_orders={}", arm.name(), sequential_batch_one.leaves, distinct_write_orders(arm, sequential_batch_one)));
    }
    let jia_sequential = Arm::OneRecordPerTransaction.records_per_fsync(sequential_batch_one);
    let bing_window = Arm::OneTransactionPerFsync.records_in_dirty_window();
    emit(&mut emitter, &format!(
        "name=verdict jia_records_seq_batch1={jia_sequential} jia_journal_over_data_bp_seq_batch1={} bing_window_records={bing_window} bing_window_kib={} e91_anchor_holds={} jia_window_over_ring_milli={}",
        journal_bytes(jia_sequential) * 10_000 / data_bytes(sequential_batch_one),
        bing_window * JOURNAL_RECORD_BYTES / 1024,
        bing_window == 1847 && bing_window * JOURNAL_RECORD_BYTES / 1024 == 7388,
        Arm::OneRecordPerTransaction.records_in_dirty_window() * JOURNAL_RECORD_BYTES * 1000 / RING_BYTES
    ));
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_capacities_are_71_and_62() {
        // 记录头 78 与 95 在 56 字节项上都给 71 项（取样点不敏感）；803 字节的项分得开：(4096 − 95) ÷ 803 = 4，(4096 − 78) ÷ 803 = 5。
        assert_eq!(items_fitting(JOURNAL_HEADER_BYTES_TODAY, 803), 4);
        assert_eq!(JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES_TODAY - 71 * ITEM_BYTES_TODAY, 25);
        assert_eq!(Arm::OneRecordPerTransaction.items_per_record(), 71);
        assert_eq!(Arm::OneTransactionPerFsync.items_per_record(), 71);
        assert_eq!(Arm::ManyTransactionsPerRecord.items_per_record(), 62);
        assert_eq!(JOURNAL_HEADER_BYTES_MULTI, 102);
    }

    #[test]
    fn records_per_fsync_follow_the_preregistered_table() {
        let expected_jia = [1, 1, 8, 2, 80, 10];
        let expected_yi = [1, 1, 1, 1, 2, 1];
        let expected_bing = [1, 1, 1, 1, 2, 1];
        for (index, load) in LOADS.iter().enumerate() {
            assert_eq!(Arm::OneRecordPerTransaction.records_per_fsync(*load), expected_jia[index], "甲 {} 批 {}", load.name, load.batch);
            assert_eq!(Arm::ManyTransactionsPerRecord.records_per_fsync(*load), expected_yi[index], "乙 {} 批 {}", load.name, load.batch);
            assert_eq!(Arm::OneTransactionPerFsync.records_per_fsync(*load), expected_bing[index], "丙 {} 批 {}", load.name, load.batch);
        }
    }

    #[test]
    fn sequential_batch_one_journal_is_twelve_and_a_half_percent_of_data_under_jia() {
        let load = LOADS[2];
        assert_eq!(journal_bytes(Arm::OneRecordPerTransaction.records_per_fsync(load)) * 10_000 / data_bytes(load), 1250);
        assert_eq!(journal_bytes(Arm::ManyTransactionsPerRecord.records_per_fsync(load)) * 10_000 / data_bytes(load), 156);
        assert_eq!(data_bytes(load), 524_288);
        assert_eq!(journal_bytes(8), 65_536);
    }

    #[test]
    fn dirty_window_records_pin_the_three_readings_including_the_e91_anchor() {
        assert_eq!(Arm::OneRecordPerTransaction.records_in_dirty_window(), 65_536);
        assert_eq!(Arm::OneRecordPerTransaction.records_in_dirty_window() * JOURNAL_RECORD_BYTES / 1024, 262_144);
        assert_eq!(Arm::ManyTransactionsPerRecord.records_in_dirty_window(), 1058);
        assert_eq!(Arm::ManyTransactionsPerRecord.records_in_dirty_window() * JOURNAL_RECORD_BYTES / 1024, 4232);
        assert_eq!(Arm::OneTransactionPerFsync.records_in_dirty_window(), 1847, "E91 的口径");
        assert_eq!(Arm::OneTransactionPerFsync.records_in_dirty_window() * JOURNAL_RECORD_BYTES / 1024, 7388, "E91 的口径");
    }

    #[test]
    fn ring_readings_and_the_dirty_capacity_pin_to_hand_arithmetic() {
        let load = LOADS[2];
        assert_eq!(SAFETY_FACTOR * worst_transaction_bytes(Arm::OneRecordPerTransaction, load) / 1024, 12);
        assert_eq!(SAFETY_FACTOR * Arm::OneRecordPerTransaction.records_in_dirty_window() * JOURNAL_RECORD_BYTES / 1024, 786_432);
        assert_eq!(Arm::OneRecordPerTransaction.records_in_dirty_window() * JOURNAL_RECORD_BYTES * 1000 / RING_BYTES, 4000);
        assert_eq!(dirty_bytes_the_ring_allows_for_jia() / 1024, 174_752);
    }

    #[test]
    fn every_data_unit_in_a_fsync_gets_its_own_write_order_except_under_bing() {
        let load = LOADS[2];
        assert_eq!(distinct_write_orders(Arm::OneRecordPerTransaction, load), 8);
        assert_eq!(distinct_write_orders(Arm::ManyTransactionsPerRecord, load), 8);
        assert_eq!(distinct_write_orders(Arm::OneTransactionPerFsync, load), 1);
    }
}
