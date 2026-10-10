//! m3-w3-r2 攻方腿（Opus）的抽象时间线模型。从 m3-w3-r1 攻方腿的模型改写（发号、回退、挂载、崩溃恢复抛弃、实例切换、清扫的条文走法照抄），
//! 候选换成这一轮的：C2a、C3r 各加「随现行那一版走的谱系记录核对」的几种定义，与第一轮正推的 I-9.30 草案（①地板、②出生代截断、③历史盖住最新根）。
//! - 管理员回退的水位 = max(cur, 环里全部读得出的根)（含被崩溃恢复抛弃的根，与 `crates/singlefs-core/src/mount.rs` 的 `readable_roots` 同一个取法）；
//!   V1 臂取目标那一版的。
//! - 历史随现行那一版走：发布从上一版带；回退那次从 cur 带，谱系追加 (T_rb, T_old, T_cur, H_cur, |log(cur)|, H_new)；恢复从所选根带。
//! - 由用户定的步全部放开穷举，只固定起点（新池新建文件之后）。
use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    Product,
    RollbackTakesTheTargetsWatermark,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mutation {
    None,
    /// 谱系那一份 (H_cur, |log(cur)|) 取 R_old 那一版的（历史本身照对）。
    LineagePairFromTheTarget,
    /// 谱系那一份写 0（T_cur = H_cur = count = 0）。
    LineagePairZero,
    /// rows_all 不记回退那次的跳升行。
    RollbackRowNotWritten,
    /// 台阶行的 H 记成这次发布之前的水位。
    RowsRecordTheWatermarkBefore,
    /// 崩溃恢复抛弃最新那条根时，新实例的历史从被抛弃的那条根带（不从所选根带）。
    MountHistoryFromTheAbandonedRoot,
    /// 双变异：回退那次历史取 R_old 的（第一轮 A）且谱系那一份也取 R_old 的。
    HistoryAndLineagePairFromTheTarget,
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
    sweep_threshold: u64,
    longest: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Lin {
    t_rb: u64,
    t_old: u64,
    t_cur: u64,
    h_cur: u64,
    count: u64,
    h_new: u64,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
struct Hist {
    rows_create: Vec<(u64, u64)>,
    rows_all: Vec<(u64, u64)>,
    log: Vec<(u64, u64)>,
    lineage: Vec<Lin>,
}

fn union_sorted(base: &[(u64, u64)], extra: &[(u64, u64)]) -> Vec<(u64, u64)> {
    let mut out: Vec<(u64, u64)> = base.iter().chain(extra.iter()).copied().collect();
    out.sort_unstable();
    out.dedup();
    out
}

impl Hist {
    fn after_publish(&self, txg: u64, wm_before: u64, wm_after: u64, new_pairs: &[(u64, u64)], row_value: u64) -> Hist {
        let mut h = self.clone();
        if !new_pairs.is_empty() {
            h.rows_create.push((txg, row_value));
        }
        if wm_after != wm_before {
            h.rows_all.push((txg, row_value));
        }
        h.log = union_sorted(&h.log, new_pairs);
        h
    }
}

#[derive(Clone, Debug)]
struct Version {
    instance: u32,
    txg: u64,
    watermark: u64,
    tree: Vec<(u64, u64)>,
    floor: u64,
    /// 实例表行 (实例, T_pub)。
    table: Vec<(u32, u64)>,
    hist: Hist,
    true_hist: Hist,
    issued: Vec<(u64, u64)>,
}

#[derive(Clone, Debug)]
struct Slot {
    version: Version,
    readable: bool,
}

#[derive(Clone)]
struct State {
    ring: Vec<Option<Slot>>,
    newest_txg_written: u64,
    newest_instance: u32,
    current: Version,
}

fn abandoned_by(table: &[(u32, u64)], instance: u32, txg: u64) -> bool {
    table.iter().any(|&(i, t)| i == instance && txg > t)
}

#[derive(Default)]
struct StepInfo {
    issued_now: Vec<(u64, u64)>,
    jump_from_abandoned_only: bool,
    jump_from_a_root_on_the_timeline: bool,
    swept: bool,
    switched: bool,
    rollback_back_to_a_former_cur: bool,
}

impl State {
    fn initial(ring_slots: usize) -> State {
        let tree = vec![(1, 3)];
        let hist = Hist::default().after_publish(3, 0, 2, &tree, 2);
        let version = Version {
            instance: 1,
            txg: 3,
            watermark: 2,
            tree: tree.clone(),
            floor: 0,
            table: Vec::new(),
            hist: hist.clone(),
            true_hist: hist,
            issued: tree,
        };
        let mut state = State { ring: vec![None; ring_slots], newest_txg_written: 0, newest_instance: 1, current: version.clone() };
        state.publish(version);
        state
    }

    fn publish(&mut self, version: Version) {
        let slots = self.ring.len();
        let slot = usize::try_from(version.txg).expect("小") % slots;
        self.newest_txg_written = self.newest_txg_written.max(version.txg);
        self.ring[slot] = Some(Slot { version: version.clone(), readable: true });
        self.current = version;
    }

    fn readable_versions(&self) -> Vec<&Version> {
        let mut out: Vec<&Version> = self.ring.iter().flatten().filter(|s| s.readable).map(|s| &s.version).collect();
        out.sort_by_key(|v| (v.txg, v.instance));
        out
    }

    fn newest_readable(&self) -> &Version {
        self.readable_versions().into_iter().last().expect("环里至少一条可读的根")
    }

    /// 一次普通发布（建、删、空发布、抬 F、清扫、写行）：树、水位、F、表给定；历史从 base 带，按变异写。
    fn next_version(
        &self,
        config: &Config,
        base: &Version,
        instance: u32,
        txg: u64,
        watermark: u64,
        tree: Vec<(u64, u64)>,
        floor: u64,
        table: Vec<(u32, u64)>,
        new_pairs: &[(u64, u64)],
    ) -> Version {
        let row_value = if config.mutation == Mutation::RowsRecordTheWatermarkBefore { base.watermark } else { watermark };
        let hist = base.hist.after_publish(txg, base.watermark, watermark, new_pairs, row_value);
        let true_hist = base.true_hist.after_publish(txg, base.watermark, watermark, new_pairs, watermark);
        Version {
            instance,
            txg,
            watermark,
            tree,
            floor,
            table,
            hist,
            true_hist,
            issued: union_sorted(&base.issued, new_pairs),
        }
    }

    fn create_in(&self, config: &Config, base: &Version, txg: u64, instance: u32, table: Vec<(u32, u64)>) -> (Version, (u64, u64)) {
        let inode = base.watermark;
        let mut tree: Vec<(u64, u64)> = base.tree.iter().copied().filter(|(n, _)| *n != inode).collect();
        tree.push((inode, txg));
        tree.sort_unstable();
        let version = self.next_version(config, base, instance, txg, inode + 1, tree, base.floor, table, &[(inode, txg)]);
        (version, (inode, txg))
    }

    fn rows_for_a_new_instance(selected: &Version, new_instance: u32) -> Vec<(u32, u64)> {
        let mut table = selected.table.clone();
        for instance in selected.instance.max(1)..new_instance {
            table.retain(|&(i, _)| i != instance);
            table.push((instance, if instance == selected.instance { selected.txg } else { 0 }));
        }
        table.sort_unstable();
        table
    }

    fn sweep(&mut self, config: &Config) -> bool {
        let table = self.current.table.clone();
        if u64::try_from(table.len()).expect("小") <= config.sweep_threshold {
            return false;
        }
        let in_ring: Vec<u32> = self.ring.iter().flatten().map(|s| s.version.instance).collect();
        let kept: Vec<(u32, u64)> = table.iter().copied().filter(|(i, _)| in_ring.contains(i)).collect();
        if kept.len() == table.len() {
            return false;
        }
        let base = self.current.clone();
        let txg = self.newest_txg_written + 1;
        let version = self.next_version(config, &base, base.instance, txg, base.watermark, base.tree.clone(), base.floor, kept, &[]);
        self.publish(version);
        true
    }

    /// 恢复：新实例写行那次发布接在 `selected` 后面；历史从 `history_base` 带（正确写法就是 selected）。
    fn recover_onto(&mut self, config: &Config, selected: &Version, history_base: &Version, txg: u64) {
        let new_instance = self.newest_instance + 1;
        self.newest_instance = new_instance;
        let table = State::rows_for_a_new_instance(selected, new_instance);
        let mut base = selected.clone();
        base.hist = history_base.hist.clone();
        let version = self.next_version(config, &base, new_instance, txg, selected.watermark, selected.tree.clone(), selected.floor, table, &[]);
        self.publish(version);
    }

    fn candidate_set(&self, current: &Version) -> Vec<Version> {
        let mut out: Vec<Version> = self
            .readable_versions()
            .into_iter()
            .filter(|v| v.txg != current.txg || v.instance != current.instance)
            .filter(|v| !abandoned_by(&current.table, v.instance, v.txg))
            .filter(|v| v.txg >= current.floor)
            .cloned()
            .collect();
        out.sort_by_key(|v| std::cmp::Reverse((v.txg, v.instance)));
        out
    }

    fn apply(&mut self, config: &Config, op: Op) -> Option<StepInfo> {
        let current = self.current.clone();
        let next_txg = self.newest_txg_written + 1;
        let mut info = StepInfo::default();
        match op {
            Op::Create => {
                let (version, issued) = self.create_in(config, &current, next_txg, current.instance, current.table.clone());
                self.publish(version);
                info.issued_now.push(issued);
            }
            Op::Delete => {
                let victim = current.tree.iter().copied().filter(|(n, _)| *n >= 2).max()?;
                let tree: Vec<(u64, u64)> = current.tree.iter().copied().filter(|p| *p != victim).collect();
                let version = self.next_version(config, &current, current.instance, next_txg, current.watermark, tree, current.floor, current.table.clone(), &[]);
                self.publish(version);
            }
            Op::Rollback(k) => {
                let candidates = self.candidate_set(&current);
                let target = candidates.get(usize::from(k) - 1)?.clone();
                let readable = self.readable_versions();
                let ring_all = readable.iter().map(|v| v.watermark).max().unwrap_or(0);
                let ring_on_the_timeline = readable
                    .iter()
                    .filter(|v| !abandoned_by(&current.table, v.instance, v.txg))
                    .map(|v| v.watermark)
                    .max()
                    .unwrap_or(0);
                info.jump_from_abandoned_only = ring_all > current.watermark && ring_on_the_timeline <= current.watermark;
                info.jump_from_a_root_on_the_timeline = ring_on_the_timeline > current.watermark;
                let watermark = match config.arm {
                    Arm::RollbackTakesTheTargetsWatermark => target.watermark,
                    Arm::Product => current.watermark.max(ring_all),
                };
                info.rollback_back_to_a_former_cur = current.true_hist.lineage.iter().any(|l| l.t_cur == target.txg);
                let make = |base_hist: &Hist, h_cur: u64, count: u64, t_cur: u64, row_jump: bool| -> Hist {
                    let mut h = base_hist.clone();
                    if row_jump && watermark != current.watermark {
                        h.rows_all.push((next_txg, watermark));
                    }
                    h.lineage.push(Lin { t_rb: next_txg, t_old: target.txg, t_cur, h_cur, count, h_new: watermark });
                    h
                };
                let cur_count = u64::try_from(current.hist.log.len()).expect("小");
                let target_count = u64::try_from(target.hist.log.len()).expect("小");
                let hist = match config.mutation {
                    Mutation::LineagePairFromTheTarget => make(&current.hist, target.watermark, target_count, current.txg, true),
                    Mutation::LineagePairZero => make(&current.hist, 0, 0, 0, true),
                    Mutation::RollbackRowNotWritten => make(&current.hist, current.watermark, cur_count, current.txg, false),
                    Mutation::HistoryAndLineagePairFromTheTarget => make(&target.hist, target.watermark, target_count, current.txg, true),
                    Mutation::None | Mutation::RowsRecordTheWatermarkBefore | Mutation::MountHistoryFromTheAbandonedRoot => {
                        make(&current.hist, current.watermark, cur_count, current.txg, true)
                    }
                };
                let true_count = u64::try_from(current.true_hist.log.len()).expect("小");
                let mut true_hist = current.true_hist.clone();
                if watermark != current.watermark {
                    true_hist.rows_all.push((next_txg, watermark));
                }
                true_hist.lineage.push(Lin { t_rb: next_txg, t_old: target.txg, t_cur: current.txg, h_cur: current.watermark, count: true_count, h_new: watermark });
                let version = Version {
                    instance: current.instance,
                    txg: next_txg,
                    watermark,
                    tree: target.tree.clone(),
                    floor: current.floor,
                    table: current.table.clone(),
                    hist,
                    true_hist,
                    issued: union_sorted(&current.issued, &target.tree),
                };
                self.publish(version);
            }
            Op::RaiseFloor => {
                if current.floor >= current.txg {
                    return None;
                }
                let version = self.next_version(config, &current, current.instance, next_txg, current.watermark, current.tree.clone(), current.txg, current.table.clone(), &[]);
                self.publish(version);
            }
            Op::Empty => {
                let version = self.next_version(config, &current, current.instance, next_txg, current.watermark, current.tree.clone(), current.floor, current.table.clone(), &[]);
                self.publish(version);
            }
            Op::Mount => {
                let selected = self.newest_readable().clone();
                self.recover_onto(config, &selected, &selected, next_txg);
                info.swept = self.sweep(config);
            }
            Op::CrashHidingTheNewestRoot => {
                let newest = self.newest_readable().clone();
                if newest.txg != current.txg || newest.instance != current.instance {
                    return None;
                }
                let slot = usize::try_from(newest.txg).expect("小") % self.ring.len();
                self.ring[slot].as_mut().expect("在环里").readable = false;
                if self.readable_versions().is_empty() {
                    return None;
                }
                let selected = self.newest_readable().clone();
                let history_base = if config.mutation == Mutation::MountHistoryFromTheAbandonedRoot { newest.clone() } else { selected.clone() };
                self.recover_onto(config, &selected, &history_base, next_txg);
                if let Some(s) = self.ring[slot].as_mut() {
                    s.readable = true;
                }
                info.swept = self.sweep(config);
            }
            Op::CrashInTheMiddleOfACreate => {
                let selected = self.newest_readable().clone();
                self.recover_onto(config, &selected, &selected, next_txg);
                info.swept = self.sweep(config);
            }
            Op::SwitchRedoingTheCreate | Op::SwitchFailingTheCreate => {
                let selected = self.newest_readable().clone();
                let new_instance = self.newest_instance + 1;
                self.newest_instance = new_instance;
                let table = State::rows_for_a_new_instance(&selected, new_instance);
                info.switched = true;
                if op == Op::SwitchRedoingTheCreate {
                    let (version, issued) = self.create_in(config, &selected, next_txg, new_instance, table);
                    self.publish(version);
                    info.issued_now.push(issued);
                } else {
                    let version = self.next_version(config, &selected, new_instance, next_txg, selected.watermark, selected.tree.clone(), selected.floor, table, &[]);
                    self.publish(version);
                }
            }
        }
        Some(info)
    }
}

const CANDIDATES: [&str; 14] = [
    "I-9.29",
    "I930/C2a-create",
    "I930/C2a-all",
    "C2aL-all:pair-cover",
    "C2aL-all:pair-eq",
    "C2aL-create:pair-cover",
    "C2aL-all:pair(Tcur,Hnew)",
    "C3r",
    "C3rs",
    "C3r:I930-3",
    "C3rL:count-cover",
    "C3rL:count-eq",
    "C2aL-strongest",
    "C3rL-strongest",
];
const N: usize = CANDIDATES.len();

fn two_births(pairs: &[(u64, u64)]) -> bool {
    pairs.windows(2).any(|w| w[0].0 == w[1].0 && w[0].1 != w[1].1)
}
fn floor_red(rows: &[(u64, u64)], wm: u64) -> bool {
    rows.iter().any(|(_, h)| *h > wm)
}
fn cut_red(rows: &[(u64, u64)], tree: &[(u64, u64)]) -> bool {
    tree.iter().any(|&(n, b)| rows.iter().any(|&(g, h)| h > n && b > g))
}
fn cover_red(rows: &[(u64, u64)], wm: u64, txg: u64) -> bool {
    wm > 2 && !rows.iter().any(|&(g, h)| h >= wm && g <= txg)
}
fn i930(rows: &[(u64, u64)], wm: u64, txg: u64, tree: &[(u64, u64)]) -> bool {
    floor_red(rows, wm) || cut_red(rows, tree) || cover_red(rows, wm, txg)
}
fn max_h_upto(rows: &[(u64, u64)], t: u64) -> u64 {
    rows.iter().filter(|(g, _)| *g <= t).map(|(_, h)| *h).max().unwrap_or(0)
}
fn log_upto(log: &[(u64, u64)], t: u64) -> u64 {
    u64::try_from(log.iter().filter(|(_, b)| *b <= t).count()).expect("小")
}

fn verdicts(state: &State) -> [bool; N] {
    let v = state.newest_readable();
    let h = &v.hist;
    let (wm, txg, tree) = (v.watermark, v.txg, &v.tree);
    let mut pairs: Vec<(u64, u64)> = tree.clone();
    for other in state.readable_versions() {
        if !abandoned_by(&v.table, other.instance, other.txg) && other.txg >= v.floor {
            pairs.extend(other.tree.iter().copied());
        }
    }
    pairs.sort_unstable();
    pairs.dedup();
    let i9_29 = two_births(&pairs);
    let c2a_create = i930(&h.rows_create, wm, txg, tree);
    let c2a_all = i930(&h.rows_all, wm, txg, tree);
    let pair_cover_all = h.lineage.iter().any(|l| max_h_upto(&h.rows_all, l.t_cur) < l.h_cur);
    let pair_eq_all = h.lineage.iter().any(|l| l.h_cur == 0 || max_h_upto(&h.rows_all, l.t_cur) != l.h_cur);
    let pair_cover_create = h.lineage.iter().any(|l| max_h_upto(&h.rows_create, l.t_cur) < l.h_cur);
    let pair_hnew = h.lineage.iter().any(|l| max_h_upto(&h.rows_all, l.t_cur) < l.h_new);
    let c3r = two_births(&h.log) || h.log.iter().any(|(n, _)| *n >= wm);
    let c3rs = c3r || tree.iter().any(|p| !h.log.contains(p));
    let log_max = h.log.iter().map(|(n, _)| *n).max().unwrap_or(0);
    let c3r_cover = wm > 2 && log_max + 1 < wm;
    let count_cover = h.lineage.iter().any(|l| log_upto(&h.log, l.t_cur) < l.count);
    let count_eq = h.lineage.iter().any(|l| l.count == 0 || log_upto(&h.log, l.t_cur) != l.count);
    [
        i9_29,
        c2a_create,
        c2a_all,
        pair_cover_all,
        pair_eq_all,
        pair_cover_create,
        pair_hnew,
        c3r,
        c3rs,
        c3r_cover,
        count_cover,
        count_eq,
        c2a_all || pair_eq_all,
        c3rs || count_eq,
    ]
}

/// 每个候选读的那一份历史被变异改过没有。
fn mutated_components(state: &State) -> [bool; N] {
    let v = state.newest_readable();
    let (h, t) = (&v.hist, &v.true_hist);
    let rc = h.rows_create != t.rows_create;
    let ra = h.rows_all != t.rows_all;
    let lg = h.log != t.log;
    // 谱系那一份按每个候选读的字段比：C2aL 读 (T_cur, H_cur)，(T_cur, H_new) 那一读法读 H_new，C3rL 读 (T_cur, count)。
    let proj = |f: &dyn Fn(&Lin) -> (u64, u64, u64)| -> bool {
        h.lineage.iter().map(|l| f(l)).collect::<Vec<_>>() != t.lineage.iter().map(|l| f(l)).collect::<Vec<_>>()
    };
    let ln_h = proj(&|l: &Lin| (l.t_rb, l.t_cur, l.h_cur));
    let ln_new = proj(&|l: &Lin| (l.t_rb, l.t_cur, l.h_new));
    let ln_n = proj(&|l: &Lin| (l.t_rb, l.t_cur, l.count));
    [false, rc, ra, ra || ln_h, ra || ln_h, rc || ln_h, ra || ln_new, lg, lg, lg, lg || ln_n, lg || ln_n, ra || ln_h, lg || ln_n]
}

const KINDS: [&str; 6] = [
    "false_positive",
    "event_missed",
    "residual_missed",
    "mutated_history_undetected",
    "false_positive_after_a_jump_from_abandoned_roots",
    "false_positive_after_a_sweep_or_switch_or_aba",
];

#[derive(Default, Clone)]
struct Tally {
    images: u64,
    legal_images: u64,
    events: u64,
    residual_images: u64,
    jumps_from_abandoned_only: u64,
    jumps_from_a_root_on_the_timeline: u64,
    sweeps: u64,
    switches: u64,
    aba_rollbacks: u64,
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

/// 一段历史走到这一步为止发生过什么（判「跳升之后」「清扫 / 切换 / A→B→A 之后」那两格用）。
#[derive(Default, Clone, Copy)]
struct Path {
    jumped_from_abandoned: bool,
    sweep_switch_or_aba: bool,
}

impl Tally {
    fn new() -> Tally {
        Tally { examples: vec![vec![None; KINDS.len()]; N], ..Tally::default() }
    }
    fn hit(&mut self, c: usize, kind: usize, seq: &[u8]) {
        self.counts[c][kind] += 1;
        if better(seq, &self.examples[c][kind]) {
            self.examples[c][kind] = Some(seq.to_vec());
        }
    }
    fn judge(&mut self, config: &Config, state: &State, step: &StepInfo, path: Path, seq: &[u8]) {
        self.images += 1;
        if step.jump_from_abandoned_only {
            self.jumps_from_abandoned_only += 1;
        }
        if step.jump_from_a_root_on_the_timeline {
            self.jumps_from_a_root_on_the_timeline += 1;
        }
        if step.swept {
            self.sweeps += 1;
        }
        if step.switched {
            self.switches += 1;
        }
        if step.rollback_back_to_a_former_cur {
            self.aba_rollbacks += 1;
        }
        let red = verdicts(state);
        let mutated = mutated_components(state);
        let v = state.newest_readable();
        let reissue = two_births(&v.issued);
        let event = step.issued_now.iter().any(|&(n, b)| v.issued.iter().any(|&(o, ob)| o == n && ob != b));
        let residual = !event && v.tree.iter().any(|&(n, b)| v.issued.iter().any(|&(o, ob)| o == n && ob < b));
        let regressed = v.issued.iter().any(|(n, _)| *n >= v.watermark);
        let legal = config.arm == Arm::Product && config.mutation == Mutation::None && !reissue && !regressed;
        if legal {
            self.legal_images += 1;
        }
        if event {
            self.events += 1;
        }
        if residual {
            self.residual_images += 1;
        }
        let across = step.issued_now.iter().any(|&(n, b)| {
            state.readable_versions().iter().any(|o| abandoned_by(&v.table, o.instance, o.txg) && o.tree.iter().any(|&(on, ob)| on == n && ob != b))
        });
        if across && !event {
            self.reissues_across_abandoned_roots += 1;
        }
        for c in 0..N {
            if legal && red[c] {
                self.hit(c, 0, seq);
                if path.jumped_from_abandoned {
                    self.hit(c, 4, seq);
                }
                if path.sweep_switch_or_aba {
                    self.hit(c, 5, seq);
                }
            }
            if event && !red[c] {
                self.hit(c, 1, seq);
            }
            if residual && !red[c] {
                self.hit(c, 2, seq);
            }
            if mutated[c] {
                self.mutated_images[c] += 1;
                if !red[c] {
                    self.hit(c, 3, seq);
                }
            }
        }
    }
}

fn explore(config: &Config, state: &State, path: Path, seq: &mut Vec<u8>, tally: &mut Tally, histories: &mut u64) {
    if seq.len() == config.longest {
        *histories += 1;
        return;
    }
    for (index, op) in Op::ALL.iter().enumerate() {
        let mut next = state.clone();
        let Some(step) = next.apply(config, *op) else { continue };
        let next_path = Path {
            jumped_from_abandoned: path.jumped_from_abandoned || step.jump_from_abandoned_only,
            sweep_switch_or_aba: path.sweep_switch_or_aba || step.swept || step.switched || step.rollback_back_to_a_former_cur,
        };
        seq.push(u8::try_from(index).expect("12 个"));
        tally.judge(config, &next, &step, next_path, seq);
        explore(config, &next, next_path, seq, tally, histories);
        seq.pop();
    }
}

fn render(v: &Version) -> String {
    let p = |list: &[(u64, u64)]| list.iter().map(|(a, b)| format!("{a}@{b}")).collect::<Vec<_>>().join(",");
    let r = |list: &[(u64, u64)]| list.iter().map(|(a, b)| format!("{a}:{b}")).collect::<Vec<_>>().join(",");
    let lin: Vec<String> = v.hist.lineage.iter().map(|l| format!("{}<-{}|cur{}:H{}:n{}:Hnew{}", l.t_rb, l.t_old, l.t_cur, l.h_cur, l.count, l.h_new)).collect();
    let table: Vec<String> = v.table.iter().map(|(i, t)| format!("{i}:{t}")).collect();
    format!(
        "newest={}:{} wm={} F={} tree=[{}] table=[{}] rows_create=[{}] rows_all=[{}] log=[{}] lineage=[{}]",
        v.instance, v.txg, v.watermark, v.floor, p(&v.tree), table.join(","), r(&v.hist.rows_create), r(&v.hist.rows_all), p(&v.hist.log), lin.join(",")
    )
}

fn trace(config: &Config, seq: &[u8]) -> Vec<String> {
    let mut state = State::initial(config.ring_slots);
    let mut lines = vec![format!("step=start {}", render(state.newest_readable()))];
    for (i, index) in seq.iter().enumerate() {
        let op = Op::ALL[usize::from(*index)];
        let info = state.apply(config, op).expect("复放");
        let red = verdicts(&state);
        let names: Vec<&str> = CANDIDATES.iter().zip(red).filter(|(_, r)| *r).map(|(n, _)| *n).collect();
        let ring: Vec<String> = state
            .ring
            .iter()
            .flatten()
            .map(|s| format!("{}:{}:wm{}{}", s.version.instance, s.version.txg, s.version.watermark, if s.readable { "" } else { "(hidden)" }))
            .collect();
        lines.push(format!(
            "step={i} op={} {} ring=[{}] issued_reissue={} jump_from_abandoned={} red=[{}]",
            op.code(),
            render(state.newest_readable()),
            ring.join(","),
            u8::from(two_births(&state.newest_readable().issued)),
            u8::from(info.jump_from_abandoned_only),
            names.join(",")
        ));
    }
    lines
}

fn label(c: &Config) -> String {
    let t = if c.sweep_threshold == u64::MAX { "never".to_string() } else { c.sweep_threshold.to_string() };
    format!("arm={:?} mutation={:?} ring={} sweep_threshold={t} longest={}", c.arm, c.mutation, c.ring_slots, c.longest)
}

fn run(config: Config) {
    let mut tally = Tally::new();
    let mut histories = 0;
    explore(&config, &State::initial(config.ring_slots), Path::default(), &mut Vec::new(), &mut tally, &mut histories);
    let l = label(&config);
    println!(
        "W3R2MODEL {l} full_length_histories={histories} images={} legal_images={} events={} residual_images={} reissues_across_abandoned_roots={} rollback_jumps_from_abandoned_roots_only={} rollback_jumps_from_a_root_on_the_timeline={} sweeps={} switches={} rollbacks_back_to_a_former_cur={}",
        tally.images, tally.legal_images, tally.events, tally.residual_images, tally.reissues_across_abandoned_roots, tally.jumps_from_abandoned_only,
        tally.jumps_from_a_root_on_the_timeline, tally.sweeps, tally.switches, tally.aba_rollbacks
    );
    for (c, name) in CANDIDATES.iter().enumerate() {
        let counts: Vec<String> = KINDS.iter().enumerate().map(|(k, kn)| format!("{kn}={}", tally.counts[c][k])).collect();
        println!("W3R2COUNT {l} candidate={name} {} mutated_images={}", counts.join(" "), tally.mutated_images[c]);
    }
    for (c, name) in CANDIDATES.iter().enumerate() {
        for (k, kn) in KINDS.iter().enumerate() {
            if let Some(seq) = &tally.examples[c][k] {
                let ops: Vec<String> = seq.iter().map(|i| Op::ALL[usize::from(*i)].code()).collect();
                println!("W3R2EXAMPLE {l} candidate={name} kind={kn} ops=[{}]", ops.join(","));
                for line in trace(&config, seq) {
                    println!("W3R2TRACE {l} candidate={name} kind={kn} {line}");
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let selection = args.first().map_or("all", String::as_str).to_string();
    let only: Option<usize> = args.get(1).map(|s| s.parse().expect("组内序号"));
    let base = Config { arm: Arm::Product, mutation: Mutation::None, ring_slots: 4, sweep_threshold: u64::MAX, longest: 7 };
    let mut configs: Vec<(&str, Config)> = Vec::new();
    for arm in [Arm::Product, Arm::RollbackTakesTheTargetsWatermark] {
        for threshold in [u64::MAX, 0] {
            configs.push(("main", Config { arm, sweep_threshold: threshold, ..base }));
        }
    }
    for mutation in [
        Mutation::LineagePairFromTheTarget,
        Mutation::LineagePairZero,
        Mutation::RollbackRowNotWritten,
        Mutation::RowsRecordTheWatermarkBefore,
        Mutation::MountHistoryFromTheAbandonedRoot,
        Mutation::HistoryAndLineagePairFromTheTarget,
    ] {
        for arm in [Arm::Product, Arm::RollbackTakesTheTargetsWatermark] {
            configs.push(("mutation", Config { arm, mutation, sweep_threshold: 0, longest: 6, ..base }));
        }
    }
    for arm in [Arm::Product, Arm::RollbackTakesTheTargetsWatermark] {
        configs.push(("ring6", Config { arm, ring_slots: 6, sweep_threshold: 0, longest: 6, ..base }));
    }
    let mut index_in_group: BTreeMap<&str, usize> = BTreeMap::new();
    for (group, config) in configs {
        let index = index_in_group.entry(group).or_insert(0);
        let this = *index;
        *index += 1;
        if (selection == "all" || selection == group) && only.is_none_or(|w| w == this) {
            run(config);
        }
    }
}
