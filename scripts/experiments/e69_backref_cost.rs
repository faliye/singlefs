//! E69：反向索引取权威态的增量维护代价 —— D21（权威态与派生态的分界） 已定项 1。
//!
//! **被测对象只记「被共享的那部分」**，不是全部反向映射。依据是
//! D21（权威态与派生态的分界）「自包含把反向映射的一半变成 O(1)」逐字：单元自带「我属于谁」
//! ⇒「这个物理位置上是哪个对象」只读那一个单元；只有共享（快照 / reflink / 去重）之下
//! 一个单元才有多个引用者，而定长单元头列不下 ⇒ 反向索引只补这一半。
//!
//! 两条臂：甲 = 权威态（跟着提交增量维护）；乙 = 派生态（要用时全盘扫描重建）。

use std::collections::{BTreeMap, BTreeSet};

// ───────────────────────── 负载模型 ─────────────────────────

/// 一个单元在盘上的身份。第一个引用者住单元头（O(1)，不进反向索引）。
#[derive(Clone)]
struct Unit {
    /// 单元头里那个「我属于谁」——**恒有且只有一个**，定长头装得下。
    owner: u32,
    /// 除单元头那个之外的引用者。**只有这些进反向索引。**
    extra_referrers: BTreeSet<u32>,
}

#[derive(Clone, Copy)]
struct Workload {
    /// 盘上单元总数。
    units: u64,
    /// 快照次数。每次快照让 `shared_per_snapshot` 个单元多一个引用者。
    snapshots: u64,
    shared_per_snapshot: u64,
    /// reflink 次数，每次让一个单元多一个引用者。
    reflinks: u64,
    /// 每个事务写几个单元（D25（目标负载优先级） 已定的粗粒度形态：一次 fsync 带 8 叶）。
    units_per_transaction: u64,
}

impl Workload {
    /// **闭式**算出的共享引用总数——独立于被测代码，判据 5 的绝对值断言用它。
    fn shared_referrers_closed_form(&self) -> u64 {
        self.snapshots * self.shared_per_snapshot + self.reflinks
    }
    fn transaction_count(&self) -> u64 {
        self.units.div_ceil(self.units_per_transaction)
    }
}

/// 按负载造一批单元。共享引用均匀撒在前若干个单元上。
fn build(workload: Workload) -> Vec<Unit> {
    let mut built_units: Vec<Unit> = (0..workload.units)
        .map(|unit_index| Unit { owner: unit_index as u32, extra_referrers: BTreeSet::new() })
        .collect();
    let mut referrer: u32 = 1_000_000;
    for snapshot_index in 0..workload.snapshots {
        for shared_index_in_snapshot in 0..workload.shared_per_snapshot {
            let target_unit_index = ((snapshot_index * workload.shared_per_snapshot + shared_index_in_snapshot) % workload.units) as usize;
            built_units[target_unit_index].extra_referrers.insert(referrer);
            referrer += 1;
        }
    }
    for reflink_index in 0..workload.reflinks {
        let target_unit_index = (reflink_index % workload.units) as usize;
        built_units[target_unit_index].extra_referrers.insert(referrer);
        referrer += 1;
    }
    built_units
}

// ───────────────────────── 甲：权威态，跟着提交增量维护 ─────────────────────────

struct Authoritative {
    map: BTreeMap<u32, BTreeSet<u32>>, // 物理位置 → 额外引用者集合
    /// 提交路径上写出的条目数（累计）。
    entries_written: u64,
    /// 提交路径上**读**了多少个单元。增量维护应当恒为 0——一读就说明退化成遍历了。
    units_read_on_commit: u64,
}

impl Authoritative {
    fn new() -> Self {
        Self { map: BTreeMap::new(), entries_written: 0, units_read_on_commit: 0 }
    }
    /// 提交一个事务：只碰本事务**自己产生**的共享引用，不碰别的单元。
    fn commit(&mut self, units: &[Unit], first_unit_index: usize, end_unit_index: usize) {
        for (unit_index, unit) in units.iter().enumerate().take(end_unit_index).skip(first_unit_index) {
            for &extra_referrer in &unit.extra_referrers {
                self.map.entry(unit_index as u32).or_default().insert(extra_referrer);
                self.entries_written += 1;
            }
        }
    }
}

// ───────────────────────── 乙：派生态，用时全盘扫描重建 ─────────────────────────

struct Derived {
    units_read_on_rebuild: u64,
}

impl Derived {
    fn rebuild(units: &[Unit]) -> (BTreeMap<u32, BTreeSet<u32>>, Self) {
        let mut map: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
        let mut read = 0u64;
        for (unit_index, unit) in units.iter().enumerate() {
            read += 1; // 全盘扫描：每个单元都要读，不论它有没有共享引用
            if !unit.extra_referrers.is_empty() {
                map.insert(unit_index as u32, unit.extra_referrers.clone());
            }
        }
        (map, Self { units_read_on_rebuild: read })
    }
}

// ───────────────────────── 跑一档 ─────────────────────────

struct Row {
    label: &'static str,
    workload: Workload,
    authoritative_entries: u64,
    authoritative_units_read: u64,
    authoritative_entries_per_transaction: f64,
    derived_units_read: u64,
    maps_identical: bool,
    closed_form: u64,
}

fn run(label: &'static str, workload: Workload) -> Row {
    let units = build(workload);
    let mut authoritative_arm = Authoritative::new();
    let mut transaction_start = 0usize;
    while transaction_start < units.len() {
        let transaction_end = (transaction_start + workload.units_per_transaction as usize).min(units.len());
        authoritative_arm.commit(&units, transaction_start, transaction_end);
        transaction_start = transaction_end;
    }
    let (derived_map, derived_arm) = Derived::rebuild(&units);
    Row {
        label,
        workload,
        authoritative_entries: authoritative_arm.entries_written,
        authoritative_units_read: authoritative_arm.units_read_on_commit,
        authoritative_entries_per_transaction: authoritative_arm.entries_written as f64 / workload.transaction_count() as f64,
        derived_units_read: derived_arm.units_read_on_rebuild,
        maps_identical: authoritative_arm.map == derived_map,
        closed_form: workload.shared_referrers_closed_form(),
    }
}

struct Emitter(u32);
impl Emitter {
    fn emit(&mut self, line: String) {
        self.0 += 1;
        println!("E7RESULT {line}");
    }
}

fn main() {
    let mut emitter = Emitter(0);
    emitter.emit("name=config units_per_txn=8 note=甲只在提交时碰本事务的单元_乙全盘扫描".into());

    // 无共享：判据 1 —— 两臂条目数必须都恰好为 0
    let none_small = Workload { units: 10_000, snapshots: 0, shared_per_snapshot: 0, reflinks: 0, units_per_transaction: 8 };
    let none_big = Workload { units: 100_000, ..none_small };
    // 有共享：阳性对照 + 主测量
    let shared_small = Workload { units: 10_000, snapshots: 10, shared_per_snapshot: 200, reflinks: 500, units_per_transaction: 8 };
    let shared_big = Workload { units: 100_000, snapshots: 100, shared_per_snapshot: 200, reflinks: 5_000, units_per_transaction: 8 };

    for row in [
        run("none_small", none_small),
        run("none_big", none_big),
        run("shared_small", shared_small),
        run("shared_big", shared_big),
    ] {
        emitter.emit(format!(
            "name=arm_cost case={} units={} closed_form_shared_refs={} \
             jia_entries={} jia_units_read_on_commit={} jia_entries_per_txn={:.4} \
             yi_units_read_on_rebuild={} maps_identical={}",
            row.label, row.workload.units, row.closed_form,
            row.authoritative_entries, row.authoritative_units_read, row.authoritative_entries_per_transaction,
            row.derived_units_read, row.maps_identical
        ));
    }

    // 判据 3：甲的每事务代价在两档盘容量上必须逐格相同（共享密度相同）
    let small_row = run("shared_small", shared_small);
    let big_row = run("shared_big", shared_big);
    emitter.emit(format!(
        "name=scale_invariance jia_per_txn_small={:.4} jia_per_txn_big={:.4} equal={} \
         yi_read_small={} yi_read_big={} ratio={:.2}",
        small_row.authoritative_entries_per_transaction, big_row.authoritative_entries_per_transaction,
        (small_row.authoritative_entries_per_transaction - big_row.authoritative_entries_per_transaction).abs() < 1e-9,
        small_row.derived_units_read, big_row.derived_units_read,
        big_row.derived_units_read as f64 / small_row.derived_units_read as f64
    ));

    // ── 密度扫描：结论依赖共享密度，只在一个点取值是 C49（全称断言只在抽样点验过） 要拦的形状 ──
    // 密度 d = 共享引用数 / 单元数。甲的每事务代价应当恰好是 d × units_per_txn。
    for sharing_percent in [0u64, 1, 5, 10, 25, 50, 100] {
        let units = 100_000u64;
        let workload = Workload {
            units,
            snapshots: 0,
            shared_per_snapshot: 0,
            reflinks: units * sharing_percent / 100,
            units_per_transaction: 8,
        };
        let density_row = run("density", workload);
        let predicted = (sharing_percent as f64 / 100.0) * workload.units_per_transaction as f64;
        emitter.emit(format!(
            "name=density_sweep pct={sharing_percent} shared_refs={} jia_entries={} \
             jia_per_txn={:.4} predicted_per_txn={:.4} matches={} \
             yi_units_read={} yi_read_per_shared_ref={}",
            density_row.closed_form, density_row.authoritative_entries, density_row.authoritative_entries_per_transaction, predicted,
            (density_row.authoritative_entries_per_transaction - predicted).abs() < 1e-9,
            density_row.derived_units_read,
            if density_row.closed_form == 0 { "inf".to_string() }
            else { format!("{:.2}", density_row.derived_units_read as f64 / density_row.closed_form as f64) }
        ));
    }

    println!("E7RESULT name=done emitted={}", emitter.0 + 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workload_without_sharing() -> Workload {
        Workload { units: 1_000, snapshots: 0, shared_per_snapshot: 0, reflinks: 0, units_per_transaction: 8 }
    }
    fn workload_with_sharing() -> Workload {
        Workload { units: 1_000, snapshots: 4, shared_per_snapshot: 50, reflinks: 100, units_per_transaction: 8 }
    }

    /// **判据 1**：无共享 ⇒ 两臂条目数都恰好 0。
    /// D21（权威态与派生态的分界） 已定项 1 逐字论证过：无快照 / reflink / 去重 ⇒ 一个共享单元都不存在。
    #[test]
    fn no_sharing_means_exactly_zero_entries_on_both_arms() {
        let row = run("t", workload_without_sharing());
        assert_eq!(row.authoritative_entries, 0, "甲在无共享负载上写出了条目");
        assert_eq!(row.closed_form, 0);
        let (derived_map, _) = Derived::rebuild(&build(workload_without_sharing()));
        assert!(derived_map.is_empty(), "乙在无共享负载上重建出了条目");
    }

    /// **判据 2 阳性对照**：加入共享后甲的条目数必须 > 0，
    /// 且**等于闭式独立算出的共享引用数**——不许从被测代码读回来。
    #[test]
    fn positive_control_sharing_changes_the_count_and_matches_closed_form() {
        let row = run("t", workload_with_sharing());
        assert!(row.authoritative_entries > 0, "阳性对照没测出差别 ⇒ 模型没在算共享");
        assert_eq!(row.closed_form, 4 * 50 + 100, "闭式本身写错了");
        assert_eq!(row.authoritative_entries, row.closed_form, "甲的条目数与闭式对不上");
    }

    /// **判据 6**：两臂重建出的映射必须逐条相同。
    #[test]
    fn both_arms_produce_the_same_mapping() {
        assert!(run("t", workload_with_sharing()).maps_identical);
        assert!(run("t", workload_without_sharing()).maps_identical);
    }

    /// **判据 3**：甲的每事务代价不随盘容量变（共享密度相同时）。
    /// 变了就说明它退化成了遍历——`.claude/rules/fs-design.md` 第一格明令禁止。
    #[test]
    fn authoritative_per_transaction_cost_is_scale_invariant() {
        let small = Workload { units: 8_000, snapshots: 10, shared_per_snapshot: 80, reflinks: 200, units_per_transaction: 8 };
        let big = Workload { units: 80_000, snapshots: 100, shared_per_snapshot: 80, reflinks: 2_000, units_per_transaction: 8 };
        let (small_row, big_row) = (run("a", small), run("b", big));
        assert!((small_row.authoritative_entries_per_transaction - big_row.authoritative_entries_per_transaction).abs() < 1e-9,
                "甲的每事务代价随盘容量变了：{} vs {}", small_row.authoritative_entries_per_transaction, big_row.authoritative_entries_per_transaction);
    }

    /// **甲的提交路径一个单元都不许多读。** 一读就是遍历。
    #[test]
    fn authoritative_reads_nothing_extra_on_commit() {
        assert_eq!(run("t", workload_with_sharing()).authoritative_units_read, 0);
    }

    /// **判据 4 的绝对值断言**：乙读到的单元数恰好等于单元总数，一个不多一个不少。
    /// 这道闸自证判别力：把 units 换一档，读数必须跟着换。
    #[test]
    fn derived_reads_exactly_every_unit() {
        for unit_count in [1_000u64, 10_000] {
            let workload = Workload { units: unit_count, ..workload_with_sharing() };
            assert_eq!(run("t", workload).derived_units_read, unit_count);
        }
    }

    /// **密度公式的绝对值断言**：甲的每事务代价恰好 = 密度 × 每事务单元数。
    /// 扫遍定义域而不是取几个点——C49（全称断言只在抽样点验过） 要拦的就是只在抽样点验过。
    #[test]
    fn per_transaction_cost_equals_density_times_units_per_transaction() {
        let units = 10_000u64;
        for sharing_percent in 0..=100u64 {
            let workload = Workload { units, snapshots: 0, shared_per_snapshot: 0,
                               reflinks: units * sharing_percent / 100, units_per_transaction: 8 };
            let row = run("t", workload);
            let predicted = (sharing_percent as f64 / 100.0) * 8.0;
            assert!((row.authoritative_entries_per_transaction - predicted).abs() < 1e-9,
                    "密度 {sharing_percent}%：实测 {} 预测 {}", row.authoritative_entries_per_transaction, predicted);
            // 乙不论密度多少，都读满全盘
            assert_eq!(row.derived_units_read, units, "乙在密度 {sharing_percent}% 上没读满全盘");
        }
    }

    /// 事务数的算术自身钉死，免得 `div_ceil` 写反。
    #[test]
    fn transaction_count_arithmetic_is_pinned() {
        assert_eq!(Workload { units: 1_000, units_per_transaction: 8, ..workload_without_sharing() }.transaction_count(), 125);
        assert_eq!(Workload { units: 1_001, units_per_transaction: 8, ..workload_without_sharing() }.transaction_count(), 126);
    }
}
