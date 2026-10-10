//! m3-w3-r1 攻方腿（Opus）的抽象时间线模型。只照条文走，不读 crates/：
//! - D8 已定项 6 射程：「时间线」= 所选根的祖先链，含管理员回退抛弃的那一段、不含崩溃恢复抛弃的那一段；
//! - D23 已定项 14：管理员回退是挂着时的一次向前发布，水位取 max(cur, 环里读得出的根)（V1 臂改成取目标那一版的）；
//!   候选集 = 环里按 cur 的实例表有效 ∧ txg ≥ F ∧ 带文件；实例切换重新读盘择根、写行、重发在飞的 checkpoint；
//! - D18 已定项 11：行只在恢复与切换时写，(i, T_pub, W)；按行判抛弃；行在根环里没有该实例的根、行数超过阈值时被清扫删掉。
//! 每一步之后对「镜像」判各候选：只读最新那条根的记录（根下的候选）、它的实例表、盘上的定长发号区（C3f 那几种）。
//! 由用户定的步（什么时候崩、回退到哪、建几个、什么时候挂载、阈值）全部放开穷举，只固定起点（新池新建文件之后）。
use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    Product,
    RollbackTakesTheTargetsWatermark,
    PublishDoesNotAddTheNewInodes,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mutation {
    None,
    /// 回退那次，根下的发号历史（C2*、C3r）与谱系里那份「回退前水位」都取 R_old 那一版的（V1 的同源错）。
    HistoryFromTheTarget,
    /// 历史只照抄这一版自己的值，不从父版本带（C2c 峰值 = 水位、C2a 只留这一代的行、C3r 只留这一版的树……）。
    HistoryCopiesThisVersion,
    /// 历史一样都不写（C1 的 M 写 0）。
    HistoryNotWritten,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Op {
    Create,
    Delete,
    Rollback(u8),
    RaiseFloor,
    Empty,
    Mount,
    CrashHidingTheNewestRoot,
    CrashInTheMiddleOfACreate,
    SwitchRedoingTheCreate,
    SwitchFailingTheCreate,
}

impl Op {
    const ALL: [Op; 12] = [
        Op::Create,
        Op::Delete,
        Op::Rollback(1),
        Op::Rollback(2),
        Op::Rollback(3),
        Op::RaiseFloor,
        Op::Empty,
        Op::Mount,
        Op::CrashHidingTheNewestRoot,
        Op::CrashInTheMiddleOfACreate,
        Op::SwitchRedoingTheCreate,
        Op::SwitchFailingTheCreate,
    ];

    fn code(self) -> String {
        match self {
            Op::Create => "create".into(),
            Op::Delete => "delete".into(),
            Op::Rollback(k) => format!("rollback{k}"),
            Op::RaiseFloor => "raiseF".into(),
            Op::Empty => "empty".into(),
            Op::Mount => "mount".into(),
            Op::CrashHidingTheNewestRoot => "crash-hide".into(),
            Op::CrashInTheMiddleOfACreate => "crash-mid-create".into(),
            Op::SwitchRedoingTheCreate => "switch-redo".into(),
            Op::SwitchFailingTheCreate => "switch-error".into(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Config {
    arm: Arm,
    mutation: Mutation,
    ring_slots: usize,
    /// 行数超过它就在挂载末尾清扫；u64::MAX 表示从不清扫（产品占位 369 在短历史里够不着）。
    sweep_threshold: u64,
    longest: usize,
}

/// 实例表一行：(实例, T_pub, M)；M 有两份读法：盘上所选那一版的水位（C1），与切换时内存里的水位（C1m）。
#[derive(Clone, PartialEq, Eq, Debug)]
struct Row {
    instance: u32,
    selected_txg: u64,
    m_disk: u64,
    m_memory: u64,
    m_true: u64,
}

/// 根下的候选记录。
#[derive(Clone, PartialEq, Eq, Debug, Default)]
struct Records {
    /// C2a：历代 (代, 水位)。
    generation_rows: Vec<(u64, u64)>,
    /// C2b：每个实例最后一代的水位。
    per_instance_last: Vec<(u32, u64)>,
    /// C2c：峰值。
    peak: u64,
    /// C2d：(峰值, 第一次到达它的代)。
    peak_pair: (u64, u64),
    /// C3r：根下只追加的 (号, 出生代)。
    log: Vec<(u64, u64)>,
    /// C4：谱系记录带的回退前水位里最大的那个。
    lineage_max: u64,
    /// 这条时间线上写过的每一行 (实例, T_pub, M)，清扫删掉的也留着：只在 true_records 里维护，给 C1k 用，不是盘上的东西。
    rows_ever: Vec<(u32, u64, u64)>,
    /// C2ay：谱系记录多带的 (cur 的 txg, 回退前水位)。谱系树随现行那一版，所以只在 true_records 里带；值按变异写（攻方腿提的改法，零轮）。
    lineage_entries: Vec<(u64, u64)>,
}

#[derive(Clone, Debug)]
struct Version {
    instance: u32,
    txg: u64,
    watermark: u64,
    tree: Vec<(u64, u64)>,
    floor: u64,
    table: Vec<Row>,
    records: Records,
    true_records: Records,
    /// 真相：这条时间线上发过的 (号, 出生代)。
    issued: Vec<(u64, u64)>,
}

#[derive(Clone, Debug)]
struct Slot {
    version: Version,
    readable: bool,
}

/// 盘上定长发号区的一条（C3f 三种读法共用）：(实例, txg, 号, 出生代)，写在那次发布的根之前、单独持久。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct FixedEntry {
    instance: u32,
    txg: u64,
    inode: u64,
    birth: u64,
}

fn union_sorted(base: &[(u64, u64)], extra: &[(u64, u64)]) -> Vec<(u64, u64)> {
    let mut out: Vec<(u64, u64)> = base.iter().chain(extra.iter()).copied().collect();
    out.sort_unstable();
    out.dedup();
    out
}

fn upsert<K: PartialEq + Copy, V: Copy>(list: &mut Vec<(K, V)>, key: K, value: V) {
    if let Some(entry) = list.iter_mut().find(|(k, _)| *k == key) {
        entry.1 = value;
    } else {
        list.push((key, value));
    }
}

/// 照写规则从父版本的记录带出这一版的记录（正确写法）。
fn extend_records(base: &Records, instance: u32, txg: u64, watermark: u64, tree: &[(u64, u64)]) -> Records {
    let mut records = base.clone();
    upsert(&mut records.generation_rows, txg, watermark);
    upsert(&mut records.per_instance_last, instance, watermark);
    if watermark > records.peak {
        records.peak = watermark;
        records.peak_pair = (watermark, txg);
    }
    records.log = union_sorted(&records.log, tree);
    records
}

fn mutated_records(
    mutation: Mutation,
    base: &Records,
    instance: u32,
    txg: u64,
    watermark: u64,
    tree: &[(u64, u64)],
) -> Records {
    match mutation {
        Mutation::None | Mutation::HistoryFromTheTarget => extend_records(base, instance, txg, watermark, tree),
        Mutation::HistoryCopiesThisVersion => Records {
            generation_rows: vec![(txg, watermark)],
            per_instance_last: vec![(instance, watermark)],
            peak: watermark,
            peak_pair: (watermark, txg),
            log: tree.to_vec(),
            lineage_max: base.lineage_max,
            rows_ever: Vec::new(),
            lineage_entries: Vec::new(),
        },
        Mutation::HistoryNotWritten => Records::default(),
    }
}

#[derive(Clone)]
struct State {
    ring: Vec<Option<Slot>>,
    newest_txg_written: u64,
    newest_instance: u32,
    current: Version,
    fixed: Vec<FixedEntry>,
    fixed_erased_by_the_sweep: Vec<FixedEntry>,
}

/// 一步做完之后要判的东西：这一步发出去的 (号, 出生代)（有就是一次发号），这一步之后的镜像就是 State 本身。
struct StepResult {
    issued_now: Vec<(u64, u64)>,
}

impl State {
    fn initial(ring_slots: usize) -> State {
        // 新池新建文件之后：实例 1，txg 3，inode 1 出生代 3，水位 2（D8 已定项 6「新池新建文件写」）。
        let tree = vec![(1, 3)];
        let records = extend_records(&Records::default(), 1, 3, 2, &tree);
        let version = Version {
            instance: 1,
            txg: 3,
            watermark: 2,
            tree: tree.clone(),
            floor: 0,
            table: Vec::new(),
            records: records.clone(),
            true_records: records,
            issued: tree,
        };
        let mut state = State {
            ring: vec![None; ring_slots],
            newest_txg_written: 0,
            newest_instance: 1,
            current: version.clone(),
            fixed: vec![FixedEntry { instance: 1, txg: 3, inode: 1, birth: 3 }],
            fixed_erased_by_the_sweep: vec![FixedEntry { instance: 1, txg: 3, inode: 1, birth: 3 }],
        };
        state.publish(version);
        state
    }

    fn publish(&mut self, version: Version) {
        let slots = self.ring.len();
        let slot = usize::try_from(version.txg).expect("txg 很小") % slots;
        self.newest_txg_written = self.newest_txg_written.max(version.txg);
        self.ring[slot] = Some(Slot { version: version.clone(), readable: true });
        self.current = version;
    }

    fn readable_versions(&self) -> Vec<&Version> {
        let mut out: Vec<&Version> = self.ring.iter().flatten().filter(|slot| slot.readable).map(|slot| &slot.version).collect();
        out.sort_by_key(|version| (version.txg, version.instance));
        out
    }

    fn newest_readable(&self) -> &Version {
        self.readable_versions().into_iter().last().expect("环里至少一条可读的根")
    }

    fn append_fixed(&mut self, entry: FixedEntry) {
        self.fixed.push(entry);
        self.fixed_erased_by_the_sweep.push(entry);
    }
}

fn abandoned_by(table: &[Row], instance: u32, txg: u64) -> bool {
    table.iter().any(|row| row.instance == instance && txg > row.selected_txg)
}

impl State {
    /// 在 `base`（内容来源）之上建一版：树、水位、F、实例表给定，记录从 `record_base` 带。
    #[allow(clippy::too_many_arguments)]
    fn next_version(
        &self,
        config: &Config,
        instance: u32,
        txg: u64,
        watermark: u64,
        tree: Vec<(u64, u64)>,
        floor: u64,
        table: Vec<Row>,
        record_base: &Records,
        true_base: &Records,
        issued: Vec<(u64, u64)>,
    ) -> Version {
        let records = mutated_records(config.mutation, record_base, instance, txg, watermark, &tree);
        let mut true_records = extend_records(true_base, instance, txg, watermark, &tree);
        for row in &table {
            let key = (row.instance, row.selected_txg, row.m_true);
            if !true_records.rows_ever.contains(&key) {
                true_records.rows_ever.push(key);
            }
        }
        Version { instance, txg, watermark, tree, floor, table, records, true_records, issued }
    }

    fn create_in(&self, config: &Config, base: &Version, txg: u64, instance: u32, table: Vec<Row>) -> (Version, (u64, u64)) {
        let inode = base.watermark;
        let mut tree: Vec<(u64, u64)> = base.tree.iter().copied().filter(|(number, _)| *number != inode).collect();
        tree.push((inode, txg));
        tree.sort_unstable();
        let watermark = match config.arm {
            Arm::PublishDoesNotAddTheNewInodes => inode,
            Arm::Product | Arm::RollbackTakesTheTargetsWatermark => inode + 1,
        };
        let issued = union_sorted(&base.issued, &[(inode, txg)]);
        let version = self.next_version(
            config, instance, txg, watermark, tree, base.floor, table, &base.records, &base.true_records, issued,
        );
        (version, (inode, txg))
    }

    /// 恢复或切换：按 `selected` 写行（[max(所选根的实例, 1), 新实例)），发新实例的第一次发布（`txg`）。
    /// `memory_watermark` 是切换时内存里的水位（C1m 那一读法写进 M），恢复时就是所选那一版的水位。
    fn rows_for_a_new_instance(&self, config: &Config, selected: &Version, new_instance: u32, memory_watermark: u64) -> Vec<Row> {
        let mut table: Vec<Row> = selected.table.clone();
        for instance in selected.instance.max(1)..new_instance {
            table.retain(|row| row.instance != instance);
            let row = if instance == selected.instance {
                Row {
                    instance,
                    selected_txg: selected.txg,
                    m_disk: if config.mutation == Mutation::HistoryNotWritten { 0 } else { selected.watermark },
                    m_memory: memory_watermark,
                    m_true: selected.watermark,
                }
            } else {
                Row { instance, selected_txg: 0, m_disk: 0, m_memory: 0, m_true: 0 }
            };
            table.push(row);
        }
        table.sort_by_key(|row| row.instance);
        table
    }

    fn sweep(&mut self, config: &Config) {
        let table = self.current.table.clone();
        if u64::try_from(table.len()).expect("小") <= config.sweep_threshold {
            return;
        }
        let in_ring: Vec<u32> = self.ring.iter().flatten().map(|slot| slot.version.instance).collect();
        let deleted: Vec<Row> = table.iter().filter(|row| !in_ring.contains(&row.instance)).cloned().collect();
        if deleted.is_empty() {
            return;
        }
        let kept: Vec<Row> = table.iter().filter(|row| in_ring.contains(&row.instance)).cloned().collect();
        let base = self.current.clone();
        let txg = self.newest_txg_written + 1;
        let version = self.next_version(
            config, base.instance, txg, base.watermark, base.tree.clone(), base.floor, kept, &base.records,
            &base.true_records, base.issued.clone(),
        );
        self.fixed_erased_by_the_sweep.retain(|entry| {
            !deleted.iter().any(|row| row.instance == entry.instance && entry.txg > row.selected_txg)
        });
        self.publish(version);
    }

    fn recover_onto(&mut self, config: &Config, selected: Version, memory_watermark: u64, txg: u64) -> Vec<Row> {
        let new_instance = self.newest_instance + 1;
        self.newest_instance = new_instance;
        let table = self.rows_for_a_new_instance(config, &selected, new_instance, memory_watermark);
        let version = self.next_version(
            config, new_instance, txg, selected.watermark, selected.tree.clone(), selected.floor, table.clone(),
            &selected.records, &selected.true_records, selected.issued.clone(),
        );
        self.publish(version);
        table
    }

    fn candidate_set(&self, current: &Version) -> Vec<Version> {
        let mut out: Vec<Version> = self
            .readable_versions()
            .into_iter()
            .filter(|version| version.txg != current.txg || version.instance != current.instance)
            .filter(|version| !abandoned_by(&current.table, version.instance, version.txg))
            .filter(|version| version.txg >= current.floor)
            .cloned()
            .collect();
        out.sort_by_key(|version| std::cmp::Reverse((version.txg, version.instance)));
        out
    }
}

impl State {
    /// 做一步；前提不满足交回 None（这条历史在这一步截掉，不往下扫）。
    fn apply(&mut self, config: &Config, op: Op) -> Option<StepResult> {
        let current = self.current.clone();
        let next_txg = self.newest_txg_written + 1;
        match op {
            Op::Create => {
                self.append_fixed(FixedEntry { instance: current.instance, txg: next_txg, inode: current.watermark, birth: next_txg });
                let (version, issued) = self.create_in(config, &current, next_txg, current.instance, current.table.clone());
                self.publish(version);
                Some(StepResult { issued_now: vec![issued] })
            }
            Op::Delete => {
                let victim = current.tree.iter().copied().filter(|(number, _)| *number >= 2).max()?;
                let tree: Vec<(u64, u64)> = current.tree.iter().copied().filter(|pair| *pair != victim).collect();
                let version = self.next_version(
                    config, current.instance, next_txg, current.watermark, tree, current.floor, current.table.clone(),
                    &current.records, &current.true_records, current.issued.clone(),
                );
                self.publish(version);
                Some(StepResult { issued_now: Vec::new() })
            }
            Op::Rollback(k) => {
                let candidates = self.candidate_set(&current);
                let target = candidates.get(usize::from(k) - 1)?.clone();
                let ring_highest = self.readable_versions().iter().map(|version| version.watermark).max().unwrap_or(0);
                let watermark = match config.arm {
                    Arm::RollbackTakesTheTargetsWatermark => target.watermark,
                    Arm::Product | Arm::PublishDoesNotAddTheNewInodes => current.watermark.max(ring_highest),
                };
                let (record_base, watermark_before) = match config.mutation {
                    Mutation::HistoryFromTheTarget => (target.records.clone(), target.watermark),
                    _ => (current.records.clone(), current.watermark),
                };
                let issued = union_sorted(&current.issued, &target.tree);
                let mut version = self.next_version(
                    config, current.instance, next_txg, watermark, target.tree.clone(), current.floor,
                    current.table.clone(), &record_base, &current.true_records, issued,
                );
                if config.mutation != Mutation::HistoryNotWritten {
                    version.records.lineage_max = current.records.lineage_max.max(watermark_before);
                }
                version.true_records.lineage_max = current.true_records.lineage_max.max(current.watermark);
                let entry = match config.mutation {
                    Mutation::None | Mutation::HistoryCopiesThisVersion => (current.txg, current.watermark),
                    Mutation::HistoryFromTheTarget => (current.txg, target.watermark),
                    Mutation::HistoryNotWritten => (0, 0),
                };
                version.true_records.lineage_entries.push(entry);
                self.publish(version);
                Some(StepResult { issued_now: Vec::new() })
            }
            Op::RaiseFloor => {
                if current.floor >= current.txg {
                    return None;
                }
                let version = self.next_version(
                    config, current.instance, next_txg, current.watermark, current.tree.clone(), current.txg,
                    current.table.clone(), &current.records, &current.true_records, current.issued.clone(),
                );
                self.publish(version);
                Some(StepResult { issued_now: Vec::new() })
            }
            Op::Empty => {
                let version = self.next_version(
                    config, current.instance, next_txg, current.watermark, current.tree.clone(), current.floor,
                    current.table.clone(), &current.records, &current.true_records, current.issued.clone(),
                );
                self.publish(version);
                Some(StepResult { issued_now: Vec::new() })
            }
            Op::Mount => {
                let selected = self.newest_readable().clone();
                self.recover_onto(config, selected.clone(), selected.watermark, next_txg);
                self.sweep(config);
                Some(StepResult { issued_now: Vec::new() })
            }
            Op::CrashHidingTheNewestRoot => {
                // 只有最新那条根、且系统配置没见证它（崩在它的根槽 FUA 之后、系统配置轮换之前）才藏得住：C554 乙罩不到的那一形。
                let newest = self.newest_readable().clone();
                if newest.txg != current.txg || newest.instance != current.instance {
                    return None;
                }
                let slot = usize::try_from(newest.txg).expect("小") % self.ring.len();
                self.ring[slot].as_mut().expect("最新那条根在环里").readable = false;
                if self.readable_versions().is_empty() {
                    return None;
                }
                let selected = self.newest_readable().clone();
                self.recover_onto(config, selected.clone(), selected.watermark, next_txg);
                if let Some(slot) = self.ring[slot].as_mut() {
                    slot.readable = true;
                }
                self.sweep(config);
                Some(StepResult { issued_now: Vec::new() })
            }
            Op::CrashInTheMiddleOfACreate => {
                // 发号区那一条已经持久、根与记录都没持久；恢复的新实例从环里最大 + 1 起，正好就是那次没成的 txg。
                self.append_fixed(FixedEntry { instance: current.instance, txg: next_txg, inode: current.watermark, birth: next_txg });
                let selected = self.newest_readable().clone();
                self.recover_onto(config, selected.clone(), selected.watermark, next_txg);
                self.sweep(config);
                Some(StepResult { issued_now: Vec::new() })
            }
            Op::SwitchRedoingTheCreate | Op::SwitchFailingTheCreate => {
                // 挂着时一次建 inode 的发布写失败（发号区那一条已持久）→ 实例切换：重新读盘择根（就是 cur）、写行、
                // 重发在飞的 checkpoint（同一个 txg，新实例）；redo 把那次建重做，error 向调用者报错、不建。
                self.append_fixed(FixedEntry { instance: current.instance, txg: next_txg, inode: current.watermark, birth: next_txg });
                let memory_watermark = match config.arm {
                    Arm::PublishDoesNotAddTheNewInodes => current.watermark,
                    Arm::Product | Arm::RollbackTakesTheTargetsWatermark => current.watermark + 1,
                };
                let selected = self.newest_readable().clone();
                let new_instance = self.newest_instance + 1;
                self.newest_instance = new_instance;
                let table = self.rows_for_a_new_instance(config, &selected, new_instance, memory_watermark);
                if op == Op::SwitchRedoingTheCreate {
                    self.append_fixed(FixedEntry { instance: new_instance, txg: next_txg, inode: selected.watermark, birth: next_txg });
                    let (version, issued) = self.create_in(config, &selected, next_txg, new_instance, table);
                    self.publish(version);
                    Some(StepResult { issued_now: vec![issued] })
                } else {
                    let version = self.next_version(
                        config, new_instance, next_txg, selected.watermark, selected.tree.clone(), selected.floor, table,
                        &selected.records, &selected.true_records, selected.issued.clone(),
                    );
                    self.publish(version);
                    Some(StepResult { issued_now: Vec::new() })
                }
            }
        }
    }
}

const N: usize = 21;
const CANDIDATES: [&str; N] = [
    "I-9.29", "I-9.6", "C1", "C1m", "C1s", "C2a", "C2as", "C2b", "C2c", "C2cs", "C2d", "C3r", "C3rs", "C3f", "C3fe", "C3fb", "C4",
    "C1k", "C2ax", "C3rx", "C2ay",
];

fn two_births(pairs: &[(u64, u64)]) -> bool {
    pairs.windows(2).any(|pair| pair[0].0 == pair[1].0 && pair[0].1 != pair[1].1)
}

fn fixed_entry_published(entry: &FixedEntry, newest: &Version) -> bool {
    if entry.instance == newest.instance {
        return entry.txg <= newest.txg;
    }
    match newest.table.iter().find(|row| row.instance == entry.instance) {
        Some(row) => entry.txg <= row.selected_txg,
        None => entry.instance < newest.instance,
    }
}

fn fixed_verdict(entries: &[FixedEntry], newest: &Version, filter: &dyn Fn(&FixedEntry) -> bool) -> bool {
    let mut pairs: Vec<(u64, u64)> = entries.iter().filter(|entry| filter(entry)).map(|entry| (entry.inode, entry.birth)).collect();
    pairs.sort_unstable();
    pairs.dedup();
    two_births(&pairs) || pairs.iter().any(|(inode, _)| *inode >= newest.watermark)
}

/// 单镜像上各候选的判定（true = 红）。只读：最新那条根、它的实例表与它下面的记录、环里读得出的根、盘上定长发号区。
fn verdicts(state: &State) -> [bool; N] {
    let newest = state.newest_readable();
    let records = &newest.records;
    let watermark = newest.watermark;
    // 现在的 I-9.29 / I-9.6：候选集（按最新那条根的实例表滤、txg ≥ F）各版的 inode 树合起来。
    let mut candidate_pairs: Vec<(u64, u64)> = newest.tree.clone();
    for version in state.readable_versions() {
        if !abandoned_by(&newest.table, version.instance, version.txg) && version.txg >= newest.floor {
            candidate_pairs.extend(version.tree.iter().copied());
        }
    }
    candidate_pairs.sort_unstable();
    candidate_pairs.dedup();
    let i9_29 = two_births(&candidate_pairs);
    let i9_6 = candidate_pairs.iter().any(|(inode, _)| *inode >= watermark);
    let c1 = newest.table.iter().any(|row| row.m_disk > watermark);
    let c1m = newest.table.iter().any(|row| row.m_memory > watermark);
    let readable = state.readable_versions();
    let c1s = c1
        || newest.table.iter().any(|row| {
            readable.iter().any(|version| version.instance == row.instance && version.txg == row.selected_txg && version.watermark != row.m_disk)
        });
    let rows_max = records.generation_rows.iter().map(|(_, value)| *value).max().unwrap_or(0);
    let generation_reissue = newest.tree.iter().any(|&(inode, birth)| {
        records.generation_rows.iter().filter(|(generation, _)| *generation < birth).map(|(_, value)| *value).max().is_some_and(|before| inode < before)
    });
    let c2a = rows_max > watermark || generation_reissue;
    let c2as = c2a || !records.generation_rows.contains(&(newest.txg, watermark));
    let c2b = records.per_instance_last.iter().any(|(_, value)| *value > watermark);
    let c2c = records.peak > watermark;
    let c2cs = records.peak != watermark;
    let (peak, peak_generation) = records.peak_pair;
    let c2d = peak > watermark || newest.tree.iter().any(|&(inode, birth)| inode < peak && birth > peak_generation);
    let c3r = two_births(&records.log) || records.log.iter().any(|(inode, _)| *inode >= watermark);
    let c3rs = c3r || newest.tree.iter().any(|pair| !records.log.contains(pair));
    let c3f = fixed_verdict(&state.fixed, newest, &|entry| fixed_entry_published(entry, newest));
    let c3fe = fixed_verdict(&state.fixed_erased_by_the_sweep, newest, &|entry| fixed_entry_published(entry, newest));
    let c3fb = fixed_verdict(&state.fixed, newest, &|entry| entry.birth <= newest.txg);
    let c4 = records.lineage_max > watermark;
    // C1k：清扫删掉的行也留着（看「行回收丢了 M」那一格用）。
    let c1k = newest.true_records.rows_ever.iter().any(|(_, _, m)| *m > watermark);
    // C2ax / C3rx：再拿环里读得出、按实例表不被抛弃的每一条根（都在这条时间线上）交叉核：它的 (代, 水位) 要在行里、它的树要在发号记录里。
    let on_the_timeline: Vec<&Version> =
        readable.iter().copied().filter(|version| !abandoned_by(&newest.table, version.instance, version.txg)).collect();
    let c2ax = c2as || on_the_timeline.iter().any(|version| !records.generation_rows.contains(&(version.txg, version.watermark)));
    let c3rx = c3rs || on_the_timeline.iter().any(|version| version.tree.iter().any(|pair| !records.log.contains(pair)));
    // C2ay：C2ax 再加「谱系里每条回退记录带的 (cur 的 txg, 回退前水位) 都在历代行里」。
    let c2ay = c2ax || newest.true_records.lineage_entries.iter().any(|entry| !records.generation_rows.contains(entry));
    [i9_29, i9_6, c1, c1m, c1s, c2a, c2as, c2b, c2c, c2cs, c2d, c3r, c3rs, c3f, c3fe, c3fb, c4, c1k, c2ax, c3rx, c2ay]
}

/// 每个候选的记录这一格被变异改过没有（与正确写法比）。
fn mutated_components(state: &State) -> [bool; N] {
    let newest = state.newest_readable();
    let (records, truth) = (&newest.records, &newest.true_records);
    let rows = newest.table.iter().any(|row| row.m_disk != row.m_true);
    let generation = records.generation_rows != truth.generation_rows;
    let per_instance = records.per_instance_last != truth.per_instance_last;
    let peak = records.peak != truth.peak;
    let pair = records.peak_pair != truth.peak_pair;
    let log = records.log != truth.log;
    let lineage = records.lineage_max != truth.lineage_max;
    [false, false, rows, false, rows, generation, generation, per_instance, peak, peak, pair, log, log, false, false, false, lineage, rows, generation, log, generation]
}

/// 真相：这条时间线上有没有同号两个出生代、最新那一版的树里有没有被重发的活对象、水位退没退回去。
struct Truth {
    reissue_on_the_timeline: bool,
    reissued_object_alive: bool,
    watermark_regressed: bool,
}

fn truth_of(state: &State) -> Truth {
    let newest = state.newest_readable();
    let issued = &newest.issued;
    // 活着的是后发的那一个（同号更早的出生代在这条时间线上发过）：被重发的对象还在最新那一版里。
    let reissued_object_alive = newest.tree.iter().any(|&(inode, birth)| issued.iter().any(|&(other, other_birth)| other == inode && other_birth < birth));
    Truth {
        reissue_on_the_timeline: two_births(issued),
        reissued_object_alive,
        watermark_regressed: issued.iter().any(|(inode, _)| *inode >= newest.watermark),
    }
}

const KINDS: [&str; 6] = [
    "false_positive",
    "event_missed",
    "event_missed_where_I-9.29_holds",
    "residual_missed",
    "mutated_history_undetected",
    "event_missed_only_because_the_sweep_deleted_the_row",
];

#[derive(Default, Clone)]
struct Tally {
    images: u64,
    legal_images: u64,
    events: u64,
    events_where_i9_29_holds: u64,
    residual_images: u64,
    /// 合法的跨被抛弃时间线重发：这一步发的号，在环里一条按最新那条根的实例表判被抛弃的根上有另一个出生代（Q3b 那一形，K2 要它出现）。
    reissues_across_abandoned_roots: u64,
    mutated_images: [u64; N],
    counts: [[u64; 6]; N],
    examples: Vec<Vec<Option<Vec<u8>>>>,
}

fn better(candidate: &[u8], kept: &Option<Vec<u8>>) -> bool {
    match kept {
        None => true,
        Some(kept) => (candidate.len(), candidate) < (kept.len(), kept.as_slice()),
    }
}

impl Tally {
    fn new() -> Tally {
        Tally { examples: vec![vec![None; KINDS.len()]; CANDIDATES.len()], ..Tally::default() }
    }

    fn hit(&mut self, candidate: usize, kind: usize, sequence: &[u8]) {
        self.counts[candidate][kind] += 1;
        if better(sequence, &self.examples[candidate][kind]) {
            self.examples[candidate][kind] = Some(sequence.to_vec());
        }
    }

    fn judge(&mut self, config: &Config, state: &State, step: &StepResult, sequence: &[u8]) {
        self.images += 1;
        let red = verdicts(state);
        let truth = truth_of(state);
        let mutated = mutated_components(state);
        let newest = state.newest_readable();
        let event = step.issued_now.iter().any(|&(inode, birth)| newest.issued.iter().any(|&(other, b)| other == inode && b != birth));
        let legal = config.mutation == Mutation::None && !truth.reissue_on_the_timeline && !truth.watermark_regressed;
        if legal {
            self.legal_images += 1;
        }
        if event {
            self.events += 1;
            if !red[0] {
                self.events_where_i9_29_holds += 1;
            }
        }
        let across_abandoned = step.issued_now.iter().any(|&(inode, birth)| {
            state.readable_versions().iter().any(|version| {
                abandoned_by(&newest.table, version.instance, version.txg)
                    && version.tree.iter().any(|&(other, b)| other == inode && b != birth)
            })
        });
        if across_abandoned && !event {
            self.reissues_across_abandoned_roots += 1;
        }
        let residual = !event && truth.reissued_object_alive;
        if residual {
            self.residual_images += 1;
        }
        for candidate in 0..CANDIDATES.len() {
            if legal && red[candidate] {
                self.hit(candidate, 0, sequence);
            }
            if event && !red[candidate] {
                self.hit(candidate, 1, sequence);
                if !red[0] {
                    self.hit(candidate, 2, sequence);
                }
            }
            if residual && !red[candidate] {
                self.hit(candidate, 3, sequence);
            }
            if event && !red[candidate] && CANDIDATES[candidate] == "C1" && red[17] {
                self.hit(candidate, 5, sequence);
            }
            if mutated[candidate] {
                self.mutated_images[candidate] += 1;
                if !red[candidate] {
                    self.hit(candidate, 4, sequence);
                }
            }
        }
    }
}

fn explore(config: &Config, state: &State, sequence: &mut Vec<u8>, tally: &mut Tally, histories: &mut u64) {
    if sequence.len() == config.longest {
        *histories += 1;
        return;
    }
    for (index, op) in Op::ALL.iter().enumerate() {
        let mut next = state.clone();
        let Some(step) = next.apply(config, *op) else { continue };
        sequence.push(u8::try_from(index).expect("12 个"));
        tally.judge(config, &next, &step, sequence);
        explore(config, &next, sequence, tally, histories);
        sequence.pop();
    }
}

fn render_version(version: &Version) -> String {
    let tree: Vec<String> = version.tree.iter().map(|(inode, birth)| format!("{inode}@{birth}")).collect();
    let rows: Vec<String> = version
        .table
        .iter()
        .map(|row| format!("{}:{}:M{}:Mm{}", row.instance, row.selected_txg, row.m_disk, row.m_memory))
        .collect();
    let log: Vec<String> = version.records.log.iter().map(|(inode, birth)| format!("{inode}@{birth}")).collect();
    format!(
        "newest={}:{} wm={} F={} tree=[{}] rows=[{}] peak={} peak_pair={}@{} lineage={} c3r_log=[{}]",
        version.instance,
        version.txg,
        version.watermark,
        version.floor,
        tree.join(","),
        rows.join(","),
        version.records.peak,
        version.records.peak_pair.0,
        version.records.peak_pair.1,
        version.records.lineage_max,
        log.join(","),
    )
}

fn render_trace(config: &Config, sequence: &[u8]) -> Vec<String> {
    let mut state = State::initial(config.ring_slots);
    let mut lines = vec![format!("step=start {}", render_version(state.newest_readable()))];
    for (step, index) in sequence.iter().enumerate() {
        let op = Op::ALL[usize::from(*index)];
        state.apply(config, op).expect("复放的历史每一步都做得成");
        let red = verdicts(&state);
        let red_names: Vec<&str> = CANDIDATES.iter().zip(red).filter(|(_, red)| *red).map(|(name, _)| *name).collect();
        let ring: Vec<String> = state
            .ring
            .iter()
            .flatten()
            .map(|slot| format!("{}:{}{}", slot.version.instance, slot.version.txg, if slot.readable { "" } else { "(hidden)" }))
            .collect();
        let fixed: Vec<String> = state.fixed.iter().map(|entry| format!("{}:{}:{}@{}", entry.instance, entry.txg, entry.inode, entry.birth)).collect();
        let truth = truth_of(&state);
        lines.push(format!(
            "step={step} op={} {} ring=[{}] fixed=[{}] truth_reissue={} truth_regressed={} red=[{}]",
            op.code(),
            render_version(state.newest_readable()),
            ring.join(","),
            fixed.join(","),
            u8::from(truth.reissue_on_the_timeline),
            u8::from(truth.watermark_regressed),
            red_names.join(","),
        ));
    }
    lines
}

fn config_label(config: &Config) -> String {
    let threshold = if config.sweep_threshold == u64::MAX { "never".to_string() } else { config.sweep_threshold.to_string() };
    format!(
        "arm={:?} mutation={:?} ring={} sweep_threshold={} longest={}",
        config.arm, config.mutation, config.ring_slots, threshold, config.longest
    )
}

fn run(config: Config) {
    let mut tally = Tally::new();
    let mut histories = 0_u64;
    let start = State::initial(config.ring_slots);
    explore(&config, &start, &mut Vec::new(), &mut tally, &mut histories);
    let label = config_label(&config);
    println!(
        "W3MODEL {label} full_length_histories={histories} images={} legal_images={} events={} events_where_I-9.29_holds={} residual_images={} reissues_across_abandoned_roots={}",
        tally.images, tally.legal_images, tally.events, tally.events_where_i9_29_holds, tally.residual_images,
        tally.reissues_across_abandoned_roots
    );
    for (candidate, name) in CANDIDATES.iter().enumerate() {
        let counts: Vec<String> = KINDS.iter().enumerate().map(|(kind, kind_name)| format!("{kind_name}={}", tally.counts[candidate][kind])).collect();
        println!("W3COUNT {label} candidate={name} {} mutated_images={}", counts.join(" "), tally.mutated_images[candidate]);
    }
    for (candidate, name) in CANDIDATES.iter().enumerate() {
        for (kind, kind_name) in KINDS.iter().enumerate() {
            if let Some(sequence) = &tally.examples[candidate][kind] {
                let ops: Vec<String> = sequence.iter().map(|index| Op::ALL[usize::from(*index)].code()).collect();
                println!("W3EXAMPLE {label} candidate={name} kind={kind_name} ops=[{}]", ops.join(","));
                for line in render_trace(&config, sequence) {
                    println!("W3TRACE {label} candidate={name} kind={kind_name} {line}");
                }
            }
        }
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let selection = arguments.first().map_or("all", String::as_str).to_string();
    let only_index: Option<usize> = arguments.get(1).map(|index| index.parse().expect("第二个参数是组内序号"));
    let mut configs: Vec<(&str, Config)> = Vec::new();
    let base = Config { arm: Arm::Product, mutation: Mutation::None, ring_slots: 4, sweep_threshold: u64::MAX, longest: 7 };
    for (arm, arm_name) in [(Arm::Product, "P"), (Arm::RollbackTakesTheTargetsWatermark, "V1"), (Arm::PublishDoesNotAddTheNewInodes, "V2")] {
        for (threshold, threshold_name) in [(u64::MAX, "never"), (0, "any")] {
            configs.push(("main", Config { arm, sweep_threshold: threshold, ..base }));
            let _ = (arm_name, threshold_name);
        }
    }
    for arm in [Arm::Product, Arm::RollbackTakesTheTargetsWatermark] {
        for mutation in [Mutation::HistoryFromTheTarget, Mutation::HistoryCopiesThisVersion, Mutation::HistoryNotWritten] {
            configs.push(("mutation", Config { arm, mutation, sweep_threshold: 0, longest: 6, ..base }));
        }
        configs.push(("ring6", Config { arm, ring_slots: 6, sweep_threshold: 0, longest: 6, ..base }));
    }
    let mut index_in_group: BTreeMap<&str, usize> = BTreeMap::new();
    for (group, config) in configs {
        let index = index_in_group.entry(group).or_insert(0);
        let this_index = *index;
        *index += 1;
        if (selection == "all" || selection == group) && only_index.is_none_or(|wanted| wanted == this_index) {
            run(config);
        }
    }
}
