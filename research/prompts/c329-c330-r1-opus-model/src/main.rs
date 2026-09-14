//! C329（写行那次发布之前推抬 F 的空发布没有检查） 与 C330（中间实例那一行的 T_pub 取所选根的 txg） 第一轮反推腿（Opus）的模型。
//! 只用 std，没有 I/O、随机源与并发，确定性。挂载历史一级：根、单元、journal 记录、实例表行、超级块里的实例代号与 txg 水位。
//! 已发布谓词与写行规则按 D18（块里携带什么信息） 第 879 行；每条候选是一组规则开关（`Rules`）。
//! 真值两样：单元的诞生 checkpoint (实例, txg) 在挂载根的时间线上（`on_timeline`）；单元是挂载根引用的现行版本（`referenced`）。
//! U1 违例 = 谓词判已发布 ∧ 不在时间线上 ∧ 没被引用；U2 违例 = 谓词判未发布 ∧ 被挂载根引用。

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnitCode {
    Data,
    Metadata,
}

#[derive(Clone, Debug)]
struct Unit {
    instance: u32,
    birth_txg: u64,
    transaction: u64,
    code: UnitCode,
    start_root_txg: u64,
    label: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Row {
    published_txg: u64,
    applied_transaction: u64,
    rollback: bool,
}

#[derive(Clone, Debug)]
struct Root {
    instance: u32,
    txg: u64,
    table: BTreeMap<u32, Row>,
    timeline: BTreeSet<(u32, u64)>,
    referenced: BTreeSet<usize>,
    accounting_unit: Option<usize>,
    covered_counter: u64,
    carried_watermark: u64,
}

#[derive(Clone, Debug)]
struct Record {
    counter: u64,
    max_data_transaction: u64,
    reconstructed: Root,
}

#[derive(Clone, Debug)]
struct Pool {
    roots: Vec<Root>,
    units: Vec<Unit>,
    records: Vec<Record>,
    superblock_instance: u32,
    superblock_watermark: u64,
}

/// C329 那一维：写行那次发布与抬 F 的空发布谁先。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RowTiming {
    TodayLiteralFRaiseFirst,
    JiaRowPublicationFirst,
    YiRowsOnEveryRoot,
    BingPredicateSide,
}

/// C330 那一维：中间实例那一行写什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IntermediateRows {
    TodaySelectedRootTxg,
    JiaZero,
}

/// C330 乙：新实例第一次发布的 txg 从哪个水位往上数（三种落点）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FirstTxgRule {
    SelectedPlusOne,
    YiSuperblockWatermark,
    YiRootCarriedWatermark,
    YiRecordScanWatermark,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PredicateRule {
    Today,
    BingNoRowMeansUnpublished,
    BingStartBeforeRowTxgUnpublished,
}

#[derive(Clone, Copy, Debug)]
struct Rules {
    name: &'static str,
    timing: RowTiming,
    intermediate: IntermediateRows,
    first_txg: FirstTxgRule,
    predicate: PredicateRule,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verdict {
    Published,
    Unpublished,
    Corrupt,
}

/// 已发布谓词（D18（块里携带什么信息） 第 879 行）：i_now = 挂载根的实例；丙的两种读法按 `PredicateRule` 切。
fn judge(rules: &Rules, mounted: &Root, unit: &Unit) -> Verdict {
    if unit.instance > mounted.instance {
        return Verdict::Corrupt;
    }
    if unit.instance == mounted.instance {
        return if unit.birth_txg <= mounted.txg { Verdict::Published } else { Verdict::Unpublished };
    }
    let Some(row) = mounted.table.get(&unit.instance) else {
        return match rules.predicate {
            PredicateRule::BingNoRowMeansUnpublished => Verdict::Unpublished,
            PredicateRule::Today | PredicateRule::BingStartBeforeRowTxgUnpublished => Verdict::Published,
        };
    };
    let birth_clause = match rules.predicate {
        PredicateRule::BingStartBeforeRowTxgUnpublished => {
            unit.birth_txg <= row.published_txg && unit.start_root_txg >= row.published_txg
        }
        PredicateRule::Today | PredicateRule::BingNoRowMeansUnpublished => unit.birth_txg <= row.published_txg,
    };
    let transaction_clause =
        unit.code == UnitCode::Data && unit.transaction != 0 && unit.transaction <= row.applied_transaction;
    if birth_clause || transaction_clause {
        Verdict::Published
    } else {
        Verdict::Unpublished
    }
}

#[derive(Default, Clone)]
struct Tally {
    u1: Vec<usize>,
    u2: Vec<usize>,
    corrupt: Vec<usize>,
    rows_not_below_mounted: Vec<u32>,
}

/// 逐单元按谓词判，与真值比；再按 I-3.8（实例表行唯一且低于挂载根） 查「行的实例代号 < 挂载根实例」。
fn evaluate(rules: &Rules, pool: &Pool, mounted: &Root) -> Tally {
    let mut tally = Tally::default();
    for (index, unit) in pool.units.iter().enumerate() {
        let referenced = mounted.referenced.contains(&index);
        let on_timeline = mounted.timeline.contains(&(unit.instance, unit.birth_txg));
        match judge(rules, mounted, unit) {
            Verdict::Published if !on_timeline && !referenced => tally.u1.push(index),
            Verdict::Unpublished if referenced => tally.u2.push(index),
            Verdict::Corrupt => tally.corrupt.push(index),
            Verdict::Published | Verdict::Unpublished => {}
        }
    }
    tally.rows_not_below_mounted =
        mounted.table.keys().copied().filter(|instance| *instance >= mounted.instance).collect();
    tally
}

fn describe_units(pool: &Pool, indices: &[usize]) -> String {
    indices
        .iter()
        .map(|index| {
            let unit = &pool.units[*index];
            format!("{}(i{},b{},n{},{:?},start{})", unit.label, unit.instance, unit.birth_txg, unit.transaction, unit.code, unit.start_root_txg)
        })
        .collect::<Vec<String>>()
        .join(",")
}

fn render_tally(pool: &Pool, tally: &Tally) -> String {
    format!(
        "U1={} [{}] U2={} [{}] corrupt={} I-3.8_rows_not_below_mounted={:?}",
        tally.u1.len(),
        describe_units(pool, &tally.u1),
        tally.u2.len(),
        describe_units(pool, &tally.u2),
        tally.corrupt.len(),
        tally.rows_not_below_mounted
    )
}

fn render_rows(table: &BTreeMap<u32, Row>) -> String {
    table
        .iter()
        .map(|(instance, row)| {
            let flag = if row.rollback { ",rollback" } else { "" };
            format!("({instance},{},{}{flag})", row.published_txg, row.applied_transaction)
        })
        .collect::<Vec<String>>()
        .join("")
}

fn mkfs_pool() -> Pool {
    let mkfs_root = Root {
        instance: 0,
        txg: 0,
        table: BTreeMap::new(),
        timeline: BTreeSet::from([(0, 0)]),
        referenced: BTreeSet::new(),
        accounting_unit: None,
        covered_counter: 0,
        carried_watermark: 0,
    };
    Pool { roots: vec![mkfs_root], units: Vec::new(), records: Vec::new(), superblock_instance: 0, superblock_watermark: 0 }
}

fn is_readable(root: &Root, unreadable: &[(u32, u64)]) -> bool {
    !unreadable.contains(&(root.instance, root.txg))
}

/// 择根（D22（单元原子性怎么合成） 已定项 7：checkpoint_txg 为主、实例代号破平局；D18 第 879 行「恒选最新可读且自证通过的根」）。
fn newest_readable(pool: &Pool, unreadable: &[(u32, u64)]) -> Root {
    pool.roots
        .iter()
        .filter(|root| is_readable(root, unreadable))
        .max_by_key(|root| (root.txg, root.instance))
        .cloned()
        .expect("mkfs 的第 0 代根在这个模型里恒读得出")
}

/// 重放（D23（journal 的角色与格式） 已定项 14 注 1：计数器严格连续、下一条的实例代号与所选根不同即停）；
/// 施加一条记录 = 换成它重建的根（D23 已定项 15）。返回重放之后的根与 W。
fn replay(pool: &Pool, selected: &Root) -> (Root, u64) {
    let mut current = selected.clone();
    let mut applied_transaction = 0;
    loop {
        let next = pool.records.iter().find(|record| record.counter == current.covered_counter + 1);
        match next {
            Some(record) if record.reconstructed.instance == current.instance => {
                applied_transaction = applied_transaction.max(record.max_data_transaction);
                current = record.reconstructed.clone();
            }
            Some(_) | None => return (current, applied_transaction),
        }
    }
}

/// 取号：max(超级块, 根环里读得出的根的实例代号) + 1（D18 第 879 行）；这个模型里取号两写都成。
fn acquire_instance(pool: &mut Pool, unreadable: &[(u32, u64)]) -> u32 {
    let from_roots = pool.roots.iter().filter(|root| is_readable(root, unreadable)).map(|root| root.instance).max().unwrap_or(0);
    let acquired = pool.superblock_instance.max(from_roots) + 1;
    pool.superblock_instance = acquired;
    acquired
}

/// 新实例第一次发布的 txg：今天 = 重放之后的根 + 1（回退 = 读得出的根里最大 + 1，D23 已定项 14）；乙的三种水位落点。
fn first_txg(rules: &Rules, pool: &Pool, unreadable: &[(u32, u64)], base_txg: u64) -> u64 {
    let readable_roots: Vec<&Root> = pool.roots.iter().filter(|root| is_readable(root, unreadable)).collect();
    let floor = match rules.first_txg {
        FirstTxgRule::SelectedPlusOne => base_txg,
        FirstTxgRule::YiSuperblockWatermark => base_txg.max(pool.superblock_watermark),
        FirstTxgRule::YiRootCarriedWatermark => {
            readable_roots.iter().map(|root| root.carried_watermark).max().unwrap_or(0).max(base_txg)
        }
        FirstTxgRule::YiRecordScanWatermark => {
            let from_records = pool.records.iter().map(|record| record.reconstructed.txg).max().unwrap_or(0);
            let from_roots = readable_roots.iter().map(|root| root.txg).max().unwrap_or(0);
            base_txg.max(pool.superblock_watermark).max(from_records).max(from_roots)
        }
    };
    floor + 1
}

/// 写行（D18 第 879 行）：给 [所选根的实例, 新实例) 每个实例写 (i, 所选根的 checkpoint_txg, 属于实例 i 的被这次重放施加的最大事务号)；
/// 回退行不许被后来的恢复覆盖；实例 0 是 mkfs、不是有效实例，不写。中间实例那一行按 C330 的候选切。
fn recovery_rows(rules: &Rules, selected: &Root, applied_transaction: u64, new_instance: u32) -> BTreeMap<u32, Row> {
    let mut table = selected.table.clone();
    for instance in selected.instance.max(1)..new_instance {
        if table.get(&instance).is_some_and(|row| row.rollback) {
            continue;
        }
        let row = if instance == selected.instance {
            Row { published_txg: selected.txg, applied_transaction, rollback: false }
        } else {
            match rules.intermediate {
                IntermediateRows::TodaySelectedRootTxg => Row { published_txg: selected.txg, applied_transaction: 0, rollback: false },
                IntermediateRows::JiaZero => Row { published_txg: 0, applied_transaction: 0, rollback: false },
            }
        };
        table.insert(instance, row);
    }
    table
}

/// 回退写 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)（D23（journal 的角色与格式） 已定项 14）。
fn rollback_rows(base_table: &BTreeMap<u32, Row>, target: &Root, new_instance: u32) -> BTreeMap<u32, Row> {
    let mut table = base_table.clone();
    table.insert(target.instance, Row { published_txg: target.txg, applied_transaction: 0, rollback: true });
    for instance in target.instance + 1..new_instance {
        table.insert(instance, Row { published_txg: 0, applied_transaction: 0, rollback: false });
    }
    table
}

/// 回退候选集：读得出、按最新根的实例表判仍有效（无行，或 T ≤ 行的 T_pub）；取候选里第二新的那一条（模型的取法）。
fn rollback_target(pool: &Pool, unreadable: &[(u32, u64)], newest: &Root) -> Option<Root> {
    let mut candidates: Vec<&Root> = pool
        .roots
        .iter()
        .filter(|root| root.instance != 0 && is_readable(root, unreadable))
        .filter(|root| newest.table.get(&root.instance).is_none_or(|row| root.txg <= row.published_txg))
        .collect();
    candidates.sort_by_key(|root| (root.txg, root.instance));
    candidates.pop();
    candidates.pop().cloned()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Reach {
    UnitsOnly,
    UnitsAndRecord,
    RootNoSuperblock,
    Complete,
}

struct Publication {
    label: String,
    data_units: u64,
    persistent_nodes: u64,
}

/// 一次发布（D16（发布语义） 已定项 7：单元 → 屏障 → 记录 → 屏障 → 根 FUA → 超级块槽），做到 `reach` 为止。
/// 每次发布写一个记账单元（盖掉父根的那一个，D16 已定项 9「空发布也是发布」）、`persistent_nodes` 个一直被引用的码 2 节点、`data_units` 个码 1。
/// 返回落了的根（没落是 None）与这次发布在内存里的根。
#[allow(clippy::too_many_arguments, reason = "一次发布的全部输入各有名字，收成结构体不增加信息")]
fn publish(
    pool: &mut Pool,
    parent: &Root,
    instance: u32,
    txg: u64,
    table: BTreeMap<u32, Row>,
    next_transaction: &mut u64,
    start_root_txg: u64,
    publication: &Publication,
    reach: Reach,
    counter: &mut u64,
) -> (Option<Root>, Root) {
    let mut new_units = vec![Unit { instance, birth_txg: txg, transaction: 0, code: UnitCode::Metadata, start_root_txg, label: format!("{}.accounting", publication.label) }];
    for node in 0..publication.persistent_nodes {
        new_units.push(Unit { instance, birth_txg: txg, transaction: 0, code: UnitCode::Metadata, start_root_txg, label: format!("{}.node{node}", publication.label) });
    }
    let mut max_data_transaction = 0;
    for _ in 0..publication.data_units {
        let transaction = *next_transaction;
        *next_transaction += 1;
        max_data_transaction = transaction;
        new_units.push(Unit { instance, birth_txg: txg, transaction, code: UnitCode::Data, start_root_txg, label: format!("{}.data{transaction}", publication.label) });
    }
    let first_index = pool.units.len();
    let mut referenced = parent.referenced.clone();
    if let Some(superseded) = parent.accounting_unit {
        referenced.remove(&superseded);
    }
    referenced.extend(first_index..first_index + new_units.len());
    let mut timeline = parent.timeline.clone();
    timeline.insert((instance, txg));
    let root = Root {
        instance,
        txg,
        table,
        timeline,
        referenced,
        accounting_unit: Some(first_index),
        covered_counter: *counter + 1,
        carried_watermark: parent.carried_watermark.max(txg),
    };
    pool.units.extend(new_units);
    if reach >= Reach::UnitsAndRecord {
        *counter += 1;
        let written_counter = *counter;
        pool.records.retain(|record| record.counter != written_counter);
        pool.records.push(Record { counter: written_counter, max_data_transaction, reconstructed: root.clone() });
    }
    let landed = if reach >= Reach::RootNoSuperblock {
        pool.roots.push(root.clone());
        Some(root.clone())
    } else {
        None
    };
    if reach == Reach::Complete {
        pool.superblock_watermark = pool.superblock_watermark.max(txg);
    }
    (landed, root)
}

#[derive(Clone)]
struct MountPlan {
    rollback_to: Option<(u32, u64)>,
    unreadable: Vec<(u32, u64)>,
    admission_short: bool,
    first: Reach,
    second: Option<Reach>,
    data_units: u64,
    persistent_nodes: u64,
    skip_rows_when_looks_clean: bool,
}

struct MountOutcome {
    selected: Root,
    base: Root,
    applied_transaction: u64,
    acquired: u32,
    first_txg: u64,
    rows: BTreeMap<u32, Row>,
    landed: Vec<Root>,
}

/// 一次可写挂载：择根 → 取号 → 重放（或回退目标）→ 写行 → 至多两次发布，做到 plan 说的地方就崩。
/// 准入不够（admission_short）时，今天的字面与丙先推一次抬 F 的空发布、它不带新行（C329 前提五），写行排第二次；
/// 甲第一次就是写行那次；乙每个根都带同一批行。
fn run_mount(rules: &Rules, pool: &mut Pool, plan: &MountPlan) -> MountOutcome {
    let selected = newest_readable(pool, &plan.unreadable);
    let acquired = acquire_instance(pool, &plan.unreadable);
    let (base, applied_transaction, rows, base_txg) = match plan.rollback_to {
        Some(target_key) => {
            let target = pool.roots.iter().find(|root| (root.instance, root.txg) == target_key).cloned().expect("回退目标在根环里");
            let ring_highest =
                pool.roots.iter().filter(|root| is_readable(root, &plan.unreadable)).map(|root| root.txg).max().unwrap_or(0);
            let rows = rollback_rows(&selected.table, &target, acquired);
            (target, 0, rows, ring_highest)
        }
        None => {
            let (reconstructed, applied) = replay(pool, &selected);
            let looks_clean = applied == 0 && plan.unreadable.is_empty();
            let rows = if plan.skip_rows_when_looks_clean && looks_clean {
                selected.table.clone()
            } else {
                recovery_rows(rules, &selected, applied, acquired)
            };
            let base_txg = reconstructed.txg;
            (reconstructed, applied, rows, base_txg)
        }
    };
    let txg = first_txg(rules, pool, &plan.unreadable, base_txg);
    let without_rows = selected.table.clone();
    let (first_table, second_table) = match rules.timing {
        RowTiming::TodayLiteralFRaiseFirst | RowTiming::BingPredicateSide if plan.admission_short => (without_rows, rows.clone()),
        RowTiming::TodayLiteralFRaiseFirst
        | RowTiming::BingPredicateSide
        | RowTiming::JiaRowPublicationFirst
        | RowTiming::YiRowsOnEveryRoot => (rows.clone(), rows.clone()),
    };
    let mut counter = base.covered_counter;
    let mut next_transaction = 1;
    let mut landed = Vec::new();
    let first_publication =
        Publication { label: format!("i{acquired}t{txg}"), data_units: plan.data_units, persistent_nodes: plan.persistent_nodes };
    let (first_root, first_in_memory) =
        publish(pool, &base, acquired, txg, first_table, &mut next_transaction, base.txg, &first_publication, plan.first, &mut counter);
    if let Some(root) = first_root {
        landed.push(root);
    }
    if let (Reach::Complete, Some(second_reach)) = (plan.first, plan.second) {
        let second_publication = Publication { label: format!("i{acquired}t{}", txg + 1), data_units: 0, persistent_nodes: 0 };
        let (second_root, _) = publish(
            pool, &first_in_memory, acquired, txg + 1, second_table, &mut next_transaction, base.txg, &second_publication, second_reach, &mut counter,
        );
        if let Some(root) = second_root {
            landed.push(root);
        }
    }
    MountOutcome { selected, base, applied_transaction, acquired, first_txg: txg, rows, landed }
}

fn plan(first: Reach, second: Option<Reach>) -> MountPlan {
    MountPlan {
        rollback_to: None,
        unreadable: Vec::new(),
        admission_short: false,
        first,
        second,
        data_units: 1,
        persistent_nodes: 0,
        skip_rows_when_looks_clean: false,
    }
}

fn render_outcome(mount_number: usize, plan: &MountPlan, outcome: &MountOutcome) -> String {
    let landed: Vec<String> = outcome.landed.iter().map(|root| format!("({},{})", root.instance, root.txg)).collect();
    format!(
        "M{mount_number} mode={} unreadable={:?} selected=({},{}) after_replay=({},{}) W={} acquired={} rows_written={} first_txg={} admission_short={} reach={:?}+{:?} landed=[{}]",
        if plan.rollback_to.is_some() { "rollback" } else { "recovery" },
        plan.unreadable,
        outcome.selected.instance,
        outcome.selected.txg,
        outcome.base.instance,
        outcome.base.txg,
        outcome.applied_transaction,
        outcome.acquired,
        render_rows(&outcome.rows),
        outcome.first_txg,
        plan.admission_short,
        plan.first,
        plan.second,
        landed.join("")
    )
}

fn render_mounted(rules: &Rules, pool: &Pool, mounted: &Root) -> String {
    format!("mounted=({},{}) table={} {}", mounted.instance, mounted.txg, render_rows(&mounted.table), render_tally(pool, &evaluate(rules, pool, mounted)))
}

fn mount_and_report(rules: &Rules, pool: &mut Pool, plan: &MountPlan, mount_number: usize, lines: &mut Vec<String>) -> MountOutcome {
    let outcome = run_mount(rules, pool, plan);
    let verdict = match outcome.landed.last() {
        Some(mounted) => render_mounted(rules, pool, mounted),
        None => "mounted=none(这次挂载一个根都没落，新行不生效)".to_string(),
    };
    lines.push(format!("  {} | {verdict}", render_outcome(mount_number, plan, &outcome)));
    outcome
}

/// 第一次可写挂载（实例 1）：暖机 (1,1) (1,2)，之后 `published_after_warm_up` 次发布各带 1 个码 1 与 1 个一直被引用的码 2 节点，都写成；
/// `in_flight` = 之后那次在飞发布做到哪、带几个码 1、几个节点，然后崩。返回最后写成的根。
fn first_mount(pool: &mut Pool, published_after_warm_up: u64, in_flight: Option<(Reach, u64, u64)>) -> Root {
    let acquired = acquire_instance(pool, &[]);
    let mut parent = pool.roots[0].clone();
    let mut counter = 0;
    let mut next_transaction = 1;
    for txg in 1..=2 + published_after_warm_up {
        let payload = u64::from(txg >= 3);
        let publication = Publication { label: format!("i{acquired}t{txg}"), data_units: payload, persistent_nodes: payload };
        let (landed, _) =
            publish(pool, &parent, acquired, txg, BTreeMap::new(), &mut next_transaction, 0, &publication, Reach::Complete, &mut counter);
        parent = landed.expect("Complete 必落");
    }
    if let Some((reach, data_units, persistent_nodes)) = in_flight {
        let txg = parent.txg + 1;
        let publication = Publication { label: format!("i{acquired}t{txg}"), data_units, persistent_nodes };
        let _ = publish(pool, &parent, acquired, txg, BTreeMap::new(), &mut next_transaction, 0, &publication, reach, &mut counter);
    }
    parent
}

fn rules(name: &'static str, timing: RowTiming, intermediate: IntermediateRows, first_txg: FirstTxgRule, predicate: PredicateRule) -> Rules {
    Rules { name, timing, intermediate, first_txg, predicate }
}

/// C329 那一维的四条臂；中间实例那一行照今天。
fn c329_rules() -> Vec<Rules> {
    use FirstTxgRule::SelectedPlusOne;
    use IntermediateRows::TodaySelectedRootTxg;
    vec![
        rules("today_literal", RowTiming::TodayLiteralFRaiseFirst, TodaySelectedRootTxg, SelectedPlusOne, PredicateRule::Today),
        rules("c329_jia", RowTiming::JiaRowPublicationFirst, TodaySelectedRootTxg, SelectedPlusOne, PredicateRule::Today),
        rules("c329_yi", RowTiming::YiRowsOnEveryRoot, TodaySelectedRootTxg, SelectedPlusOne, PredicateRule::Today),
        rules("c329_bing", RowTiming::BingPredicateSide, TodaySelectedRootTxg, SelectedPlusOne, PredicateRule::BingNoRowMeansUnpublished),
    ]
}

/// C330 那一维的六条臂；写行次序一律照 C329 甲（写行那次是本实例第一次发布），与这一维正交。
fn c330_rules() -> Vec<Rules> {
    use RowTiming::JiaRowPublicationFirst as Timing;
    use IntermediateRows::{JiaZero, TodaySelectedRootTxg as TodayRows};
    vec![
        rules("today", Timing, TodayRows, FirstTxgRule::SelectedPlusOne, PredicateRule::Today),
        rules("c330_jia", Timing, JiaZero, FirstTxgRule::SelectedPlusOne, PredicateRule::Today),
        rules("c330_yi_superblock", Timing, TodayRows, FirstTxgRule::YiSuperblockWatermark, PredicateRule::Today),
        rules("c330_yi_root_carried", Timing, TodayRows, FirstTxgRule::YiRootCarriedWatermark, PredicateRule::Today),
        rules("c330_yi_record_scan", Timing, TodayRows, FirstTxgRule::YiRecordScanWatermark, PredicateRule::Today),
        rules("c330_bing_start", Timing, TodayRows, FirstTxgRule::SelectedPlusOne, PredicateRule::BingStartBeforeRowTxgUnpublished),
    ]
}

/// C329 前提五：M1 在飞的 (1,4) 只落了单元就崩；M2 准入不够，第一个根落了、第二次发布只落了单元就崩；M3、M4 普通挂载。
fn c329_premise_five(rules: &Rules) -> Vec<String> {
    let mut pool = mkfs_pool();
    let mut lines = vec![format!("C329_PREMISE5 rules={}", rules.name)];
    let last = first_mount(&mut pool, 1, Some((Reach::UnitsOnly, 1, 1)));
    lines.push(format!("  M1 acquired=1 landed=(1,1)(1,2)(1,3) last=({},{}) in_flight=(1,4) units_only crash", last.instance, last.txg));
    let mut second = plan(Reach::Complete, Some(Reach::UnitsOnly));
    second.admission_short = true;
    mount_and_report(rules, &mut pool, &second, 2, &mut lines);
    let normal = plan(Reach::Complete, Some(Reach::Complete));
    mount_and_report(rules, &mut pool, &normal, 3, &mut lines);
    mount_and_report(rules, &mut pool, &normal, 4, &mut lines);
    lines
}

/// C329 换故障：管理员回退代替普通挂载。M1 写成 (1,1)..(1,5)；M2 回退到 (1,3)、准入不够，第一个根落了、第二次只落了单元就崩。
fn c329_rollback_form(rules: &Rules) -> Vec<String> {
    let mut pool = mkfs_pool();
    let mut lines = vec![format!("C329_ROLLBACK rules={}", rules.name)];
    first_mount(&mut pool, 3, None);
    lines.push("  M1 acquired=1 landed=(1,1)..(1,5) crash".to_string());
    let mut second = plan(Reach::Complete, Some(Reach::UnitsOnly));
    second.rollback_to = Some((1, 3));
    second.admission_short = true;
    mount_and_report(rules, &mut pool, &second, 2, &mut lines);
    let normal = plan(Reach::Complete, Some(Reach::Complete));
    mount_and_report(rules, &mut pool, &normal, 3, &mut lines);
    mount_and_report(rules, &mut pool, &normal, 4, &mut lines);
    lines
}

/// C329 换故障：「非干净结束」没有判别子。M1 在飞的 (1,4) 只落了单元、没有记录，盘上的根与记录看着与干净结束一样；
/// `skip` = 实现按「所选根之后没有记录、没有读不出的根」判成干净、不写行（最弱读法）；不 skip = 每次都写行（收严）。
fn c329_looks_clean(rules: &Rules, skip: bool) -> Vec<String> {
    let mut pool = mkfs_pool();
    let mut lines = vec![format!("C329_LOOKS_CLEAN rules={} skip_rows_when_looks_clean={skip}", rules.name)];
    first_mount(&mut pool, 1, Some((Reach::UnitsOnly, 1, 1)));
    let mut normal = plan(Reach::Complete, Some(Reach::Complete));
    normal.skip_rows_when_looks_clean = skip;
    for mount_number in 2..=4 {
        mount_and_report(rules, &mut pool, &normal, mount_number, &mut lines);
    }
    lines
}

/// 丙「干净结束也写行」若写在它自己的最后一次发布里：挂载根 (1,4) 的表里就有实例 1 的行。
fn c329_bing_clean_end_row(rules: &Rules) -> String {
    let mut pool = mkfs_pool();
    let last = first_mount(&mut pool, 1, None);
    let mut counter = last.covered_counter;
    let mut next_transaction = 2;
    let table = BTreeMap::from([(1, Row { published_txg: last.txg + 1, applied_transaction: 1, rollback: false })]);
    let publication = Publication { label: "i1t4.clean_end".to_string(), data_units: 0, persistent_nodes: 0 };
    let (landed, _) = publish(&mut pool, &last, 1, last.txg + 1, table, &mut next_transaction, 0, &publication, Reach::Complete, &mut counter);
    let mounted = landed.expect("Complete 必落");
    format!("C329_BING_CLEAN_END_ROW rules={} {}", rules.name, render_mounted(rules, &pool, &mounted))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeptTransactions {
    OnlyInMemory,
    InRebuildState,
}

/// C329 换故障：实例切换代替恢复。M2 写成 (2,4)(2,5) 之后，发布中的 checkpoint 6 两个码 1 落了、固定点写失败 ⇒ 切换到实例 3
/// （D23（journal 的角色与格式） 已定项 14：写行 (旧实例, 最后发布的 txg, 被重发的那个 checkpoint 里的最大事务号)、号 ≤ W 的事务照旧）。
/// 准入不够：今天的字面与丙先推不带行的抬 F 空发布（txg 6），乙先推带行的；甲第一次就是重发。第一个根落了、第二次只落了单元就崩。
/// `kept` = 号 ≤ W 的单元在不在重建态里（D28（挂载期承诺量） 已定项 3「重建态从所选根 + 施加到 W 的记录来」有记录才在）。
fn c329_switch_form(rules: &Rules, kept: KeptTransactions) -> Vec<String> {
    let mut pool = mkfs_pool();
    let mut lines = vec![format!("C329_SWITCH rules={} kept_transactions={kept:?}", rules.name)];
    first_mount(&mut pool, 1, None);
    let second = mount_and_report(rules, &mut pool, &plan(Reach::Complete, Some(Reach::Complete)), 2, &mut lines);
    let last = second.landed.last().cloned().expect("两次都写成");
    let mut counter = last.covered_counter;
    let mut next_transaction = 2;
    let failed_txg = last.txg + 1;
    let before = pool.units.len();
    let failed = Publication { label: format!("i2t{failed_txg}.failed"), data_units: 2, persistent_nodes: 0 };
    let _ = publish(&mut pool, &last, 2, failed_txg, last.table.clone(), &mut next_transaction, second.base.txg, &failed, Reach::UnitsOnly, &mut counter);
    let kept_units: Vec<usize> = (before..pool.units.len()).filter(|index| pool.units[*index].code == UnitCode::Data).collect();
    let switch_w = next_transaction - 1;
    let new_instance = acquire_instance(&mut pool, &[]);
    let mut rows = last.table.clone();
    rows.insert(2, Row { published_txg: last.txg, applied_transaction: switch_w, rollback: false });
    let mut with_kept = last.clone();
    with_kept.referenced.extend(kept_units.iter().copied());
    let rebuild = match kept {
        KeptTransactions::OnlyInMemory => last.clone(),
        KeptTransactions::InRebuildState => with_kept.clone(),
    };
    let mut counter_new = last.covered_counter;
    let mut transactions_new = 1;
    let (first_parent, first_table, first_label, second_is_resend) = match rules.timing {
        RowTiming::TodayLiteralFRaiseFirst | RowTiming::BingPredicateSide => (rebuild, last.table.clone(), "fraise", true),
        RowTiming::YiRowsOnEveryRoot => (rebuild, rows.clone(), "fraise", true),
        RowTiming::JiaRowPublicationFirst => (with_kept.clone(), rows.clone(), "resend", false),
    };
    let first_publication = Publication { label: format!("i{new_instance}t{failed_txg}.{first_label}"), data_units: 0, persistent_nodes: 0 };
    let (first_root, first_in_memory) = publish(
        &mut pool, &first_parent, new_instance, failed_txg, first_table, &mut transactions_new, last.txg, &first_publication, Reach::Complete, &mut counter_new,
    );
    let mut second_parent = first_in_memory;
    if second_is_resend {
        second_parent.referenced.extend(kept_units.iter().copied());
    }
    let second_label = if second_is_resend { "resend" } else { "warmup" };
    let second_publication = Publication { label: format!("i{new_instance}t{}.{second_label}", failed_txg + 1), data_units: 0, persistent_nodes: 0 };
    let _ = publish(
        &mut pool, &second_parent, new_instance, failed_txg + 1, rows, &mut transactions_new, last.txg, &second_publication, Reach::UnitsOnly, &mut counter_new,
    );
    let mounted = first_root.expect("Complete 必落");
    lines.push(format!(
        "  M2' switch old=2 new={new_instance} switch_row=(2,{},{switch_w}) kept=[{}] first={first_label} second={second_label}(units_only) crash | {}",
        last.txg,
        describe_units(&pool, &kept_units),
        render_mounted(rules, &pool, &mounted)
    ));
    let normal = plan(Reach::Complete, Some(Reach::Complete));
    mount_and_report(rules, &mut pool, &normal, 3, &mut lines);
    mount_and_report(rules, &mut pool, &normal, 4, &mut lines);
    lines
}

/// 挂载根下每个实例 2 / 3 的单元的谓词输入并排（丙：判别子看不看得见）。
fn predicate_inputs(rules: &Rules, pool: &Pool, mounted: &Root) -> Vec<String> {
    pool.units
        .iter()
        .enumerate()
        .filter(|(_, unit)| unit.instance == 2 || unit.instance == 3)
        .map(|(index, unit)| {
            let row = mounted.table.get(&unit.instance).map(|row| (row.published_txg, row.applied_transaction));
            format!(
                "    INPUTS {} code={:?} b={} start={} n={} row(T_pub,W)={:?} | verdict={:?} referenced={} on_timeline={}",
                unit.label, unit.code, unit.birth_txg, unit.start_root_txg, unit.transaction, row,
                judge(rules, mounted, unit), mounted.referenced.contains(&index), mounted.timeline.contains(&(unit.instance, unit.birth_txg))
            )
        })
        .collect()
}

/// C330 前提六（`rollback_at_second` = 管理员回退同形）：M2 选 (1,3)（或回退到 (1,1)），第一个根 (2,4) 落到 `second_first_reach`、
/// 暖机第二次只落了单元就崩；M3 那条 (2,4) 暂时读不出，第一次发布只落了单元就崩；M4、M5 全部读得出。
fn c330_premise_six(rules: &Rules, second_first_reach: Reach, rollback_at_second: bool) -> Vec<String> {
    let mut pool = mkfs_pool();
    let mut lines = vec![format!("C330_PREMISE6 rules={} m2_first_reach={second_first_reach:?} rollback_at_m2={rollback_at_second}", rules.name)];
    first_mount(&mut pool, 1, Some((Reach::UnitsOnly, 1, 1)));
    let mut second = plan(second_first_reach, Some(Reach::UnitsOnly));
    second.persistent_nodes = 1;
    if rollback_at_second {
        second.rollback_to = Some((1, 1));
    }
    let second_outcome = mount_and_report(rules, &mut pool, &second, 2, &mut lines);
    let second_root = second_outcome.landed.first().map(|root| (root.instance, root.txg)).expect("第一个根落了");
    let mut third = plan(Reach::UnitsOnly, None);
    third.persistent_nodes = 1;
    third.unreadable = vec![second_root];
    mount_and_report(rules, &mut pool, &third, 3, &mut lines);
    let normal = plan(Reach::Complete, Some(Reach::Complete));
    let fourth = mount_and_report(rules, &mut pool, &normal, 4, &mut lines);
    if let Some(mounted) = fourth.landed.first() {
        lines.extend(predicate_inputs(rules, &pool, mounted));
    }
    mount_and_report(rules, &mut pool, &normal, 5, &mut lines);
    lines
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SwitchRowReading {
    /// 切换那句「写行 (旧实例, 最后发布的 txg, 被重发的那个 checkpoint 里的最大事务号)」按字面给旧实例——第二次切换时旧实例是中间实例；
    /// 所选根那个实例按批量规则取「被这次重放施加的」= 0（切换没有记录可施加）。
    LiteralOldInstance,
    /// W 给号 ≤ W 的事务所属的实例（所选根那个实例），中间实例 0。
    OwnerOfKeptTransactions,
}

/// C330 换故障：切换里再切换。M2 写成 (2,4)(2,5)；发布中的 checkpoint 6 两个码 1（事务 2、3）落了、固定点写失败 ⇒ 切到 3；
/// 开放的 checkpoint 7 里事务 4 的码 1 已落。实例 3 重发 6：记账单元落了、另一个固定点写失败 ⇒ 再切到 4；实例 3 已把开放的事务
/// 重做成自己的事务 1（码 1 落了）。实例 4 重发 6（照旧号 ≤ W 的事务 2、3）并写成，开放的 7 按新写序重做后写成；M3、M4 普通挂载。
fn c330_consecutive_switch(rules: &Rules, reading: SwitchRowReading) -> Vec<String> {
    let mut pool = mkfs_pool();
    let mut lines = vec![format!("C330_CONSECUTIVE_SWITCH rules={} reading={reading:?}", rules.name)];
    first_mount(&mut pool, 1, None);
    let second = mount_and_report(rules, &mut pool, &plan(Reach::Complete, Some(Reach::Complete)), 2, &mut lines);
    let last = second.landed.last().cloned().expect("两次都写成");
    let publishing_txg = last.txg + 1;
    let mut counter = last.covered_counter;
    let mut transactions_two = 2;
    let before = pool.units.len();
    let publishing = Publication { label: format!("i2t{publishing_txg}.publishing"), data_units: 2, persistent_nodes: 0 };
    let (_, publishing_memory) = publish(
        &mut pool, &last, 2, publishing_txg, last.table.clone(), &mut transactions_two, second.base.txg, &publishing, Reach::UnitsOnly, &mut counter,
    );
    let kept: Vec<usize> = (before..pool.units.len()).filter(|index| pool.units[*index].code == UnitCode::Data).collect();
    let kept_w = transactions_two - 1;
    let open = Publication { label: format!("i2t{}.open", publishing_txg + 1), data_units: 1, persistent_nodes: 0 };
    let _ = publish(
        &mut pool, &publishing_memory, 2, publishing_txg + 1, last.table.clone(), &mut transactions_two, second.base.txg, &open, Reach::UnitsOnly, &mut counter,
    );
    let third_instance = acquire_instance(&mut pool, &[]);
    let mut with_kept = last.clone();
    with_kept.referenced.extend(kept.iter().copied());
    let mut counter_three = last.covered_counter;
    let mut transactions_three = 1;
    let resend_three = Publication { label: format!("i{third_instance}t{publishing_txg}.resend"), data_units: 0, persistent_nodes: 0 };
    let (_, resend_three_memory) = publish(
        &mut pool, &with_kept, third_instance, publishing_txg, last.table.clone(), &mut transactions_three, last.txg, &resend_three, Reach::UnitsOnly, &mut counter_three,
    );
    let redo_three = Publication { label: format!("i{third_instance}t{}.redo", publishing_txg + 1), data_units: 1, persistent_nodes: 0 };
    let _ = publish(
        &mut pool, &resend_three_memory, third_instance, publishing_txg + 1, last.table.clone(), &mut transactions_three, last.txg, &redo_three, Reach::UnitsOnly, &mut counter_three,
    );
    let fourth_instance = acquire_instance(&mut pool, &[]);
    let (selected_w, literal_intermediate) = match reading {
        SwitchRowReading::LiteralOldInstance => (0, Row { published_txg: last.txg, applied_transaction: kept_w, rollback: false }),
        SwitchRowReading::OwnerOfKeptTransactions => (kept_w, Row { published_txg: last.txg, applied_transaction: 0, rollback: false }),
    };
    let intermediate_row = match rules.intermediate {
        IntermediateRows::TodaySelectedRootTxg => literal_intermediate,
        IntermediateRows::JiaZero => Row { published_txg: 0, applied_transaction: 0, rollback: false },
    };
    let mut rows = last.table.clone();
    rows.insert(2, Row { published_txg: last.txg, applied_transaction: selected_w, rollback: false });
    rows.insert(third_instance, intermediate_row);
    let mut counter_four = last.covered_counter;
    let mut transactions_four = 1;
    let resend_four = Publication { label: format!("i{fourth_instance}t{publishing_txg}.resend"), data_units: 0, persistent_nodes: 0 };
    let (_, resent_memory) = publish(
        &mut pool, &with_kept, fourth_instance, publishing_txg, rows.clone(), &mut transactions_four, last.txg, &resend_four, Reach::Complete, &mut counter_four,
    );
    let redo_four = Publication { label: format!("i{fourth_instance}t{}.redo", publishing_txg + 1), data_units: 1, persistent_nodes: 0 };
    let (redone, _) = publish(
        &mut pool, &resent_memory, fourth_instance, publishing_txg + 1, rows.clone(), &mut transactions_four, last.txg, &redo_four, Reach::Complete, &mut counter_four,
    );
    let mounted = redone.expect("Complete 必落");
    lines.push(format!(
        "  M2'' switch 2->{third_instance}->{fourth_instance} kept=[{}] rows_written_by_second_switch={} | {}",
        describe_units(&pool, &kept),
        render_rows(&rows),
        render_mounted(rules, &pool, &mounted)
    ));
    let normal = plan(Reach::Complete, Some(Reach::Complete));
    mount_and_report(rules, &mut pool, &normal, 3, &mut lines);
    mount_and_report(rules, &mut pool, &normal, 4, &mut lines);
    lines
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReplayedMetadata {
    /// 恢复的第一次发布只重写记账单元，重放施加进来的码 2 节点照旧被引用（D23 已定项 15「换四个字段、不需要树语义」的直读）。
    AdoptedAsIs,
    /// 恢复的第一次发布把重放施加进来的码 2 节点全部重写（D18 第 879 行「所选根之后的固定点单元一律未发布，恢复实例重写它们」的直读）。
    RewrittenByRecovery,
    /// 所选根那个实例的行 T_pub 取重放之后那个根的 txg（改条款的读法）。
    RowTxgAfterReplay,
}

/// 重放施加进来的码 2 / 3 单元：M1 在飞的 (1,4) 带 2 个码 1 与 3 个节点，记录落了、根没落就崩；M2 选 (1,3)、重放 (1,4)，W = 3。
fn replayed_metadata_scenario(rules: &Rules, handling: ReplayedMetadata) -> Vec<String> {
    let mut pool = mkfs_pool();
    let mut lines = vec![format!("REPLAYED_METADATA rules={} handling={handling:?}", rules.name)];
    first_mount(&mut pool, 1, Some((Reach::UnitsAndRecord, 2, 3)));
    lines.push("  M1 acquired=1 landed=(1,1)(1,2)(1,3) in_flight=(1,4) data2+nodes3 record_landed root_not crash".to_string());
    let selected = newest_readable(&pool, &[]);
    let acquired = acquire_instance(&mut pool, &[]);
    let (reconstructed, applied) = replay(&pool, &selected);
    let mut rows = recovery_rows(rules, &selected, applied, acquired);
    if handling == ReplayedMetadata::RowTxgAfterReplay {
        rows.insert(selected.instance, Row { published_txg: reconstructed.txg, applied_transaction: applied, rollback: false });
    }
    let replayed_nodes: Vec<usize> = reconstructed
        .referenced
        .iter()
        .copied()
        .filter(|index| {
            let unit = &pool.units[*index];
            unit.code == UnitCode::Metadata && unit.birth_txg == reconstructed.txg && Some(*index) != reconstructed.accounting_unit
        })
        .collect();
    let mut parent = reconstructed.clone();
    let rewritten = if handling == ReplayedMetadata::RewrittenByRecovery {
        parent.referenced.retain(|index| !replayed_nodes.contains(index));
        u64::try_from(replayed_nodes.len()).expect("几个节点")
    } else {
        0
    };
    let mut counter = reconstructed.covered_counter;
    let mut transactions = 1;
    let txg = reconstructed.txg + 1;
    let first = Publication { label: format!("i{acquired}t{txg}.rowpub"), data_units: 0, persistent_nodes: rewritten };
    let (_, first_memory) =
        publish(&mut pool, &parent, acquired, txg, rows.clone(), &mut transactions, reconstructed.txg, &first, Reach::Complete, &mut counter);
    let second = Publication { label: format!("i{acquired}t{}.warmup", txg + 1), data_units: 0, persistent_nodes: 0 };
    let (second_root, _) = publish(
        &mut pool, &first_memory, acquired, txg + 1, rows.clone(), &mut transactions, reconstructed.txg, &second, Reach::Complete, &mut counter,
    );
    let mounted = second_root.expect("Complete 必落");
    lines.push(format!(
        "  M2 selected=({},{}) after_replay=({},{}) W={applied} acquired={acquired} rows_written={} replayed_nodes=[{}] row_publication_new_units=accounting1+rewritten{rewritten} | {}",
        selected.instance,
        selected.txg,
        reconstructed.instance,
        reconstructed.txg,
        render_rows(&rows),
        describe_units(&pool, &replayed_nodes),
        render_mounted(rules, &pool, &mounted)
    ));
    let normal = plan(Reach::Complete, Some(Reach::Complete));
    mount_and_report(rules, &mut pool, &normal, 3, &mut lines);
    mount_and_report(rules, &mut pool, &normal, 4, &mut lines);
    lines
}

/// U3：甲让写行那次发布走切换预留（D28（挂载期承诺量） 已定项 3），按两块盘、每块盘的 16 KiB 块数算；
/// 暖机要几次发布才覆盖两块盘，按根环区域 = txg mod 3、区域归属 0 / 1 / 0（D16（发布语义） 已定项 8 引的 D2 已定项 7）。
fn reservation_arithmetic() -> Vec<String> {
    let switch_count: u64 = 3;
    let region_count: u64 = 3;
    let chain_blocks_per_disk_per_switch: u64 = 2;
    let mut lines = Vec::new();
    for c_max in [4u64, 9] {
        let per_switch = chain_blocks_per_disk_per_switch + region_count * c_max;
        let reserved = switch_count * per_switch;
        let row_only = chain_blocks_per_disk_per_switch + c_max;
        let row_and_warm_up = chain_blocks_per_disk_per_switch + region_count * c_max;
        lines.push(format!(
            "RESERVATION c_max={c_max} per_disk: per_switch={per_switch} reserved(N_switch={switch_count})={reserved} | jia_row_publication_only={row_only} remaining={} full_switches_left={} | jia_row_publication_plus_warm_up_worst={row_and_warm_up} remaining={} full_switches_left={}",
            reserved - row_only,
            (reserved - row_only) / per_switch,
            reserved - row_and_warm_up,
            (reserved - row_and_warm_up) / per_switch
        ));
    }
    let disk_of_region = [0usize, 1, 0];
    for first_region in 0..region_count {
        let mut covered = [false, false];
        let mut publications = 0u64;
        let mut txg = first_region;
        while !(covered[0] && covered[1]) {
            covered[disk_of_region[usize::try_from(txg % region_count).expect("小于 3")]] = true;
            publications += 1;
            txg += 1;
        }
        lines.push(format!("WARM_UP first_txg_mod_3={first_region} publications_to_cover_both_disks={publications}"));
    }
    lines
}

/// 枚举（C330 那一维）：M1 之后连着四次可写挂载，每次选普通恢复或管理员回退（有候选才选）、本次挂载暂时读不出 0 / 1 / 2 条最新的根、
/// 做到哪一步；每次挂载有根落了就判一次，四次之后按「下一次挂载的第一个根」再判一次。每次发布带 1 个码 1。
const ENUMERATED_MOUNTS: usize = 4;
const STAGES: [(Reach, Option<Reach>); 5] = [
    (Reach::UnitsOnly, None),
    (Reach::UnitsAndRecord, None),
    (Reach::RootNoSuperblock, None),
    (Reach::Complete, Some(Reach::UnitsOnly)),
    (Reach::Complete, Some(Reach::Complete)),
];

#[derive(Default)]
struct EnumerationResult {
    histories: u64,
    evaluations: u64,
    u1_units: u64,
    u2_units: u64,
    histories_with_u1: u64,
    histories_with_u2: u64,
    i38_violations: u64,
    fewest_faults_u1: Option<(u32, Vec<String>)>,
    fewest_faults_u2: Option<(u32, Vec<String>)>,
}

fn newest_landed(pool: &Pool, count: usize) -> Vec<(u32, u64)> {
    let mut keys: Vec<(u64, u32)> = pool.roots.iter().filter(|root| root.instance != 0).map(|root| (root.txg, root.instance)).collect();
    keys.sort_unstable();
    keys.iter().rev().take(count).map(|(txg, instance)| (*instance, *txg)).collect()
}

fn record(result: &mut EnumerationResult, tally: &Tally, seen: (bool, bool), faults: u32, script: &[String], line: &str) -> (bool, bool) {
    result.evaluations += 1;
    result.u1_units += u64::try_from(tally.u1.len()).expect("小");
    result.u2_units += u64::try_from(tally.u2.len()).expect("小");
    result.i38_violations += u64::from(!tally.rows_not_below_mounted.is_empty());
    let history = || script.iter().cloned().chain(std::iter::once(line.to_string())).collect::<Vec<String>>();
    if !tally.u1.is_empty() && result.fewest_faults_u1.as_ref().is_none_or(|(fewest, _)| faults < *fewest) {
        result.fewest_faults_u1 = Some((faults, history()));
    }
    if !tally.u2.is_empty() && result.fewest_faults_u2.as_ref().is_none_or(|(fewest, _)| faults < *fewest) {
        result.fewest_faults_u2 = Some((faults, history()));
    }
    (seen.0 || !tally.u1.is_empty(), seen.1 || !tally.u2.is_empty())
}

fn explore(rules: &Rules, pool: &Pool, depth: usize, faults: u32, script: &mut Vec<String>, seen: (bool, bool), result: &mut EnumerationResult) {
    if depth == ENUMERATED_MOUNTS {
        let mut next = pool.clone();
        let outcome = run_mount(rules, &mut next, &plan(Reach::RootNoSuperblock, None));
        let root = outcome.landed.first().cloned().expect("RootNoSuperblock 必落根");
        let tally = evaluate(rules, &next, &root);
        let line = format!("    next_mount_first_root {}", render_mounted(rules, &next, &root));
        let final_seen = record(result, &tally, seen, faults, script, &line);
        result.histories += 1;
        result.histories_with_u1 += u64::from(final_seen.0);
        result.histories_with_u2 += u64::from(final_seen.1);
        return;
    }
    for rollback in [false, true] {
        for unreadable_count in 0..3usize {
            let unreadable = newest_landed(pool, unreadable_count);
            if unreadable.len() < unreadable_count {
                continue;
            }
            let rollback_to = if rollback {
                let newest = newest_readable(pool, &unreadable);
                match rollback_target(pool, &unreadable, &newest) {
                    Some(target) => Some((target.instance, target.txg)),
                    None => continue,
                }
            } else {
                None
            };
            for (first, second) in STAGES {
                let mount_plan = MountPlan {
                    rollback_to,
                    unreadable: unreadable.clone(),
                    admission_short: false,
                    first,
                    second,
                    data_units: 1,
                    persistent_nodes: 0,
                    skip_rows_when_looks_clean: false,
                };
                let mut next = pool.clone();
                let outcome = run_mount(rules, &mut next, &mount_plan);
                let mount_faults = faults + u32::try_from(unreadable.len()).expect("至多 2") + 1;
                let mut line = format!("    {}", render_outcome(depth + 2, &mount_plan, &outcome));
                let mut now_seen = seen;
                if let Some(mounted) = outcome.landed.last() {
                    let tally = evaluate(rules, &next, mounted);
                    line.push_str(&format!(" | mounted=({},{}) table={} {}", mounted.instance, mounted.txg, render_rows(&mounted.table), render_tally(&next, &tally)));
                    now_seen = record(result, &tally, seen, mount_faults, script, &line);
                }
                script.push(line);
                explore(rules, &next, depth + 1, mount_faults, script, now_seen, result);
                script.pop();
            }
        }
    }
}

fn main() {
    println!("CONFIG model=c329_c330_mount_history enumerated_mounts_after_first={ENUMERATED_MOUNTS} stages={} faults=transient_unreadable_roots+one_crash_per_mount", STAGES.len());
    for rules in c329_rules() {
        let mut lines = c329_premise_five(&rules);
        lines.extend(c329_rollback_form(&rules));
        for kept in [KeptTransactions::OnlyInMemory, KeptTransactions::InRebuildState] {
            lines.extend(c329_switch_form(&rules, kept));
        }
        for skip in [true, false] {
            lines.extend(c329_looks_clean(&rules, skip));
        }
        lines.iter().for_each(|line| println!("{line}"));
    }
    println!("{}", c329_bing_clean_end_row(&c329_rules()[3]));
    for rules in c330_rules() {
        let mut lines = Vec::new();
        for (second_first_reach, rollback_at_second) in [(Reach::RootNoSuperblock, false), (Reach::Complete, false), (Reach::Complete, true)] {
            lines.extend(c330_premise_six(&rules, second_first_reach, rollback_at_second));
        }
        for reading in [SwitchRowReading::LiteralOldInstance, SwitchRowReading::OwnerOfKeptTransactions] {
            lines.extend(c330_consecutive_switch(&rules, reading));
        }
        for handling in [ReplayedMetadata::AdoptedAsIs, ReplayedMetadata::RewrittenByRecovery, ReplayedMetadata::RowTxgAfterReplay] {
            lines.extend(replayed_metadata_scenario(&rules, handling));
        }
        lines.iter().for_each(|line| println!("{line}"));
    }
    reservation_arithmetic().iter().for_each(|line| println!("{line}"));
    let mut base = mkfs_pool();
    first_mount(&mut base, 1, Some((Reach::UnitsOnly, 1, 0)));
    for rules in c330_rules() {
        let mut result = EnumerationResult::default();
        explore(&rules, &base, 0, 0, &mut Vec::new(), (false, false), &mut result);
        println!(
            "ENUM rules={} histories={} evaluations={} u1_units={} u2_units={} histories_with_u1={} histories_with_u2={} evaluations_with_I-3.8_violation={}",
            rules.name, result.histories, result.evaluations, result.u1_units, result.u2_units, result.histories_with_u1, result.histories_with_u2, result.i38_violations
        );
        for (kind, example) in [("U1", &result.fewest_faults_u1), ("U2", &result.fewest_faults_u2)] {
            if let Some((faults, history)) = example {
                println!("ENUM_EXAMPLE rules={} kind={kind} faults={faults}", rules.name);
                history.iter().for_each(|line| println!("{line}"));
            }
        }
    }
}
