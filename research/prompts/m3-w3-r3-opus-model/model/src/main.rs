//! m3-w3-r3 攻方腿（Opus）的抽象时间线模型。从头重写，不拷前两轮的源码，不依赖 `crates/`。
//! 条文走法（发号、删、回退、抬 F、空发布、挂载、崩溃恢复抛弃、建到一半崩溃、切换、清扫）按 D23 已定项 14 与前两轮攻方模型文档里的字面重写，
//! 第一组配置（verify）用来独立重算前两轮报出的几个数。
//! 新加的维：
//! - 回退取水位的五种规则（Y1 维持 / Y1 只取没被抛弃的根 / 去掉 cur 内存项的两种 / V1 取目标的），回退那一刻 cur 的根读不出、实例表读不出；
//! - 两个可写头（克隆），候选按头分与不分；
//! - journal（每次发布一条记录，新实例第一条记录接在读得出的最大号之后），新实例第一个 txg 的改坏臂。
use std::collections::BTreeMap;

type Pair = (u64, u64);

/// 变异：有过一次回退之后，字面 C2a 的台阶行（rows_create）不再写（「把那份历史写错或不写」那一类，只给 K2 容许跳升那一判用）。
static ROWS_CREATE_STOP_AFTER_A_ROLLBACK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WmRule {
    /// 今天的条款与代码：max(cur 内存里的, 环里全部读得出的根)。
    MaxCurAllRing,
    /// Y1 第二种取法：max(cur 内存里的, 环里读得出、按 cur 的实例表没被抛弃的根)。
    MaxCurKeptRing,
    /// 去掉 cur 内存项，只取环里全部读得出的根（C143 第三轮之前的取法）。
    MaxAllRingOnly,
    /// 去掉 cur 内存项，只取环里读得出、没被抛弃的根。
    MaxKeptRingOnly,
    /// V1：取目标那一版的。
    Target,
    /// 两个头时只有头 0 取目标那一版的（V1 只坏一个头）。
    TargetHead0Only,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TableFallback {
    /// 今天的代码：cur 的实例表读不出就拒回退。
    Refuse,
    /// 读不出时当全部没被抛弃（退回 Y1 维持那一取法）。
    AllKept,
    /// 读不出时环里一条都不取，只取 cur 内存里的。
    CurOnly,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RecoveryWm {
    Selected,
    /// V3：恢复时新实例的水位比所选根的少 1（只在挂载、崩溃恢复里）。
    SelectedMinusOne,
    /// V3 照第二轮辩方腿的写法：少 1、不低于 2，挂载、崩溃恢复与报错的切换里都少。
    SelectedMinusOneNotBelowTwoAlsoSwitchError,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FirstTxg {
    /// max(环里读得出的根, journal 里读得出的记录) + 1（`mount.rs` 的 `first_txg_of_new_instance`）。
    Correct,
    /// 改坏：所选根 + 1。
    SelectedPlusOne,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CloneFrom {
    /// 从 cur 那一版克隆。
    Current,
    /// 从回退候选集里最新的那一版（较旧的快照）克隆。
    OlderVersion,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CloneWm {
    /// 新头的水位取快照那一版的。
    Snapshot,
    /// 新头的水位取 origin 头现行的。
    OriginCurrent,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Op {
    Create(u8),
    Delete,
    Rollback(u8),
    RaiseFloor,
    Empty,
    Mount,
    CrashHide,
    CrashMidCreate,
    SwitchRedo,
    SwitchError,
    /// 回退 1，回退那一刻 cur 的根槽读不出（瞬时）。
    RollbackCurUnreadable,
    /// 回退 1，回退那一刻 cur 的实例表读不出（瞬时）。
    RollbackTableUnreadable,
    /// 崩溃，下一次挂载时最新那条根与它的记录都读不出（瞬时，系统配置没见证它）。
    CrashHideRootAndRecord,
    /// 建第二个可写头。
    Clone,
}

impl Op {
    fn code(self) -> String {
        match self {
            Op::Create(0) => "create".into(),
            Op::Create(h) => format!("create{h}"),
            Op::Delete => "delete".into(),
            Op::Rollback(k) => format!("rollback{k}"),
            Op::RaiseFloor => "raiseF".into(),
            Op::Empty => "empty".into(),
            Op::Mount => "mount".into(),
            Op::CrashHide => "crash-hide".into(),
            Op::CrashMidCreate => "crash-mid-create".into(),
            Op::SwitchRedo => "switch-redo".into(),
            Op::SwitchError => "switch-error".into(),
            Op::RollbackCurUnreadable => "rollback1-cur-unreadable".into(),
            Op::RollbackTableUnreadable => "rollback1-table-unreadable".into(),
            Op::CrashHideRootAndRecord => "crash-hide-root-and-record".into(),
            Op::Clone => "clone".into(),
        }
    }
}

/// 前两轮模型的十二个字母，次序相同（独立重算时历史条数要对得上）。
const R2_OPS: [Op; 12] = [
    Op::Create(0),
    Op::Delete,
    Op::Rollback(1),
    Op::Rollback(2),
    Op::Rollback(3),
    Op::RaiseFloor,
    Op::Empty,
    Op::Mount,
    Op::CrashHide,
    Op::CrashMidCreate,
    Op::SwitchRedo,
    Op::SwitchError,
];

#[derive(Clone, Debug)]
struct Config {
    group: &'static str,
    rows_create_stop_after_a_rollback: bool,
    wm_rule: WmRule,
    table_fallback: TableFallback,
    recovery_wm: RecoveryWm,
    first_txg: FirstTxg,
    clone_from: CloneFrom,
    clone_wm: CloneWm,
    clone_writes_a_row: bool,
    ring: usize,
    journal: usize,
    sweep_threshold: u64,
    longest: usize,
    ops: Vec<Op>,
}

/// 一个可写头在一版里的东西：水位、inode 树，与各候选随根的历史；`issued` 是真相（这条时间线上这个头发过的 (号, 出生代)）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct Head {
    wm: u64,
    tree: Vec<Pair>,
    /// C2aL：每次水位变了的发布留 (g, H)。
    rows_all: Vec<(u64, u64)>,
    /// 字面 C2a：只在建 inode 的发布留 (g, H)。
    rows_create: Vec<(u64, u64)>,
    /// C3r / C3rL：(号, 出生代)。
    log: Vec<Pair>,
    issued: Vec<Pair>,
    /// 克隆头从快照那一版继承的谱系值 (T_rb, origin 头的 W) 与 C2i 条目 (M, g)（C4g-inh、C2i-inh 用；头 0 恒空）。
    inherited_lineage: Vec<(u64, u64)>,
    inherited_c2i: Vec<(u64, u64)>,
}

/// 谱系记录（全池一份，D8 已定项 6 射程：谱系树写 0）：(T_rb, T_old) 之外按头带回退之前 cur 的水位、cur 的发号记录条数、回退之后的水位。
#[derive(Clone, Debug, PartialEq, Eq)]
struct Lin {
    t_rb: u64,
    t_old: u64,
    t_cur: u64,
    h_cur: Vec<u64>,
    count: Vec<u64>,
    h_new: Vec<u64>,
}

#[derive(Clone, Debug)]
struct Ver {
    inst: u32,
    txg: u64,
    floor: u64,
    table: Vec<(u32, u64)>,
    heads: Vec<Head>,
    lineage: Vec<Lin>,
    /// C2i：每 (实例, 头) 一条 (最高水位 M, 第一次到它的 txg g)。
    c2i: Vec<(u32, usize, u64, u64)>,
    /// 这一版那条记录的 jsn。
    jsn: u64,
    /// 墓碑树 (号, 出生代, 死亡 txg)：随现行那一版、不回收；只记头 0（删只在头 0）。
    tombs: Vec<(u64, u64, u64)>,
}

#[derive(Clone, Debug)]
struct Slot {
    v: Ver,
    readable: bool,
}

#[derive(Clone, Copy, Debug)]
struct Rec {
    inst: u32,
    txg: u64,
    jsn: u64,
    readable: bool,
}

#[derive(Clone)]
struct State {
    ring: Vec<Option<Slot>>,
    journal: Vec<Option<Rec>>,
    newest_instance: u32,
    cur: Ver,
    /// 发布过（根写成了）的每一版建过的 (头, 号, 出生代)，不分时间线（判新实例第一个 txg 撞没撞上用）。
    ever: Vec<(usize, u64, u64)>,
}

fn union_sorted(base: &[Pair], extra: &[Pair]) -> Vec<Pair> {
    let mut out: Vec<Pair> = base.iter().chain(extra.iter()).copied().collect();
    out.sort_unstable();
    out.dedup();
    out
}

fn abandoned_by(table: &[(u32, u64)], inst: u32, txg: u64) -> bool {
    table.iter().any(|&(i, t)| i == inst && txg > t)
}

fn rows_for_a_new_instance(selected: &Ver, new_instance: u32) -> Vec<(u32, u64)> {
    let mut table = selected.table.clone();
    for i in selected.inst.max(1)..new_instance {
        table.retain(|&(x, _)| x != i);
        table.push((i, if i == selected.inst { selected.txg } else { 0 }));
    }
    table.sort_unstable();
    table
}

/// 这一次发布之后每个头的新样子：(新水位, 新树, 这次新建的 (号, 出生代))。
struct HeadChange {
    wm: u64,
    tree: Vec<Pair>,
    new_pairs: Vec<Pair>,
}

#[derive(Default, Clone)]
struct StepInfo {
    issued_now: Vec<(usize, Pair)>,
    rollback: bool,
    jump_from_abandoned_only: bool,
    kept_ring_above_cur: bool,
    rollback_below_cur: bool,
    swept: bool,
    switched: bool,
    first_txg_collides: bool,
}

impl State {
    fn initial(config: &Config) -> State {
        let head = Head {
            wm: 2,
            tree: vec![(1, 3)],
            rows_all: vec![(3, 2)],
            rows_create: vec![(3, 2)],
            log: vec![(1, 3)],
            issued: vec![(1, 3)],
            inherited_lineage: Vec::new(),
            inherited_c2i: Vec::new(),
        };
        let v = Ver { inst: 1, txg: 3, floor: 0, table: Vec::new(), heads: vec![head], lineage: Vec::new(), c2i: vec![(1, 0, 2, 3)], jsn: 0, tombs: Vec::new() };
        let mut s = State { ring: vec![None; config.ring], journal: vec![None; config.journal], newest_instance: 1, cur: v.clone(), ever: vec![(0, 1, 3)] };
        s.publish(v);
        s
    }

    fn publish(&mut self, v: Ver) {
        let slots = self.ring.len();
        let slot = usize::try_from(v.txg).expect("小") % slots;
        let j = self.journal.len();
        self.journal[usize::try_from(v.jsn).expect("小") % j] = Some(Rec { inst: v.inst, txg: v.txg, jsn: v.jsn, readable: true });
        for (h, head) in v.heads.iter().enumerate() {
            for &(n, b) in &head.tree {
                if b == v.txg && !self.ever.contains(&(h, n, b)) {
                    self.ever.push((h, n, b));
                }
            }
        }
        self.ring[slot] = Some(Slot { v: v.clone(), readable: true });
        self.cur = v;
    }

    fn readable(&self) -> Vec<&Ver> {
        let mut out: Vec<&Ver> = self.ring.iter().flatten().filter(|s| s.readable).map(|s| &s.v).collect();
        out.sort_by_key(|v| (v.txg, v.inst));
        out
    }

    fn newest_readable(&self) -> Option<&Ver> {
        self.readable().into_iter().last()
    }

    fn highest_readable_txg(&self) -> u64 {
        let ring = self.ring.iter().flatten().filter(|s| s.readable).map(|s| s.v.txg).max().unwrap_or(0);
        let rec = self.journal.iter().flatten().filter(|r| r.readable).map(|r| r.txg).max().unwrap_or(0);
        ring.max(rec)
    }

    fn highest_readable_jsn(&self) -> u64 {
        self.journal.iter().flatten().filter(|r| r.readable).map(|r| r.jsn).max().unwrap_or(0)
    }

    /// 从 base 发一版：历史随 base 走，按 changes 更新。
    fn derive(base: &Ver, inst: u32, txg: u64, jsn: u64, floor: u64, table: Vec<(u32, u64)>, changes: Vec<HeadChange>) -> Ver {
        let mut heads = Vec::with_capacity(changes.len());
        let mut c2i = base.c2i.clone();
        for (h, ch) in changes.into_iter().enumerate() {
            let b = &base.heads[h];
            let mut head = b.clone();
            if ch.wm != b.wm {
                head.rows_all.push((txg, ch.wm));
            }
            if !ch.new_pairs.is_empty() && !(ROWS_CREATE_STOP_AFTER_A_ROLLBACK.load(std::sync::atomic::Ordering::Relaxed) && !base.lineage.is_empty()) {
                head.rows_create.push((txg, ch.wm));
            }
            head.log = union_sorted(&b.log, &ch.new_pairs);
            head.issued = union_sorted(&b.issued, &ch.new_pairs);
            head.wm = ch.wm;
            head.tree = ch.tree;
            update_c2i(&mut c2i, inst, h, ch.wm, txg);
            heads.push(head);
        }
        Ver { inst, txg, floor, table, heads, lineage: base.lineage.clone(), c2i, jsn, tombs: base.tombs.clone() }
    }
}

fn update_c2i(c2i: &mut Vec<(u32, usize, u64, u64)>, inst: u32, h: usize, wm: u64, txg: u64) {
    match c2i.iter_mut().find(|e| e.0 == inst && e.1 == h) {
        Some(e) => {
            if wm > e.2 {
                e.2 = wm;
                e.3 = txg;
            }
        }
        None => c2i.push((inst, h, wm, txg)),
    }
}

fn same_heads(v: &Ver) -> Vec<HeadChange> {
    v.heads.iter().map(|h| HeadChange { wm: h.wm, tree: h.tree.clone(), new_pairs: Vec::new() }).collect()
}

fn create_change(base: &Ver, h: usize, txg: u64) -> (Vec<HeadChange>, Pair) {
    let mut changes = same_heads(base);
    let inode = base.heads[h].wm;
    let mut tree: Vec<Pair> = base.heads[h].tree.iter().copied().filter(|(n, _)| *n != inode).collect();
    tree.push((inode, txg));
    tree.sort_unstable();
    changes[h] = HeadChange { wm: inode + 1, tree, new_pairs: vec![(inode, txg)] };
    (changes, (inode, txg))
}

impl State {
    fn sweep(&mut self, config: &Config) -> bool {
        let table = self.cur.table.clone();
        if u64::try_from(table.len()).expect("小") <= config.sweep_threshold {
            return false;
        }
        let in_ring: Vec<u32> = self.ring.iter().flatten().map(|s| s.v.inst).collect();
        let kept: Vec<(u32, u64)> = table.iter().copied().filter(|(i, _)| in_ring.contains(i)).collect();
        if kept.len() == table.len() {
            return false;
        }
        let base = self.cur.clone();
        let v = State::derive(&base, base.inst, base.txg + 1, base.jsn + 1, base.floor, kept, same_heads(&base));
        self.publish(v);
        true
    }

    fn first_txg(&self, config: &Config, selected: &Ver) -> u64 {
        match config.first_txg {
            FirstTxg::Correct => self.highest_readable_txg() + 1,
            FirstTxg::SelectedPlusOne => selected.txg + 1,
        }
    }

    fn recover_onto(&mut self, config: &Config, selected: &Ver) {
        let new_instance = self.newest_instance + 1;
        self.newest_instance = new_instance;
        let table = rows_for_a_new_instance(selected, new_instance);
        let txg = self.first_txg(config, selected);
        let jsn = self.highest_readable_jsn() + 1;
        let changes: Vec<HeadChange> = selected
            .heads
            .iter()
            .map(|h| HeadChange {
                wm: match config.recovery_wm {
                    RecoveryWm::Selected => h.wm,
                    RecoveryWm::SelectedMinusOne => h.wm - 1,
                    RecoveryWm::SelectedMinusOneNotBelowTwoAlsoSwitchError => h.wm.saturating_sub(1).max(2),
                },
                tree: h.tree.clone(),
                new_pairs: Vec::new(),
            })
            .collect();
        let v = State::derive(selected, new_instance, txg, jsn, selected.floor, table, changes);
        self.publish(v);
    }

    fn candidates(&self, cur: &Ver, cur_slot_unreadable: bool) -> Vec<Ver> {
        let mut out: Vec<Ver> = self
            .readable()
            .into_iter()
            .filter(|v| !(v.txg == cur.txg && v.inst == cur.inst))
            .filter(|v| !abandoned_by(&cur.table, v.inst, v.txg))
            .filter(|v| v.txg >= cur.floor)
            .cloned()
            .collect();
        let _ = cur_slot_unreadable;
        out.sort_by_key(|v| std::cmp::Reverse((v.txg, v.inst)));
        out
    }

    fn rollback(&mut self, config: &Config, k: usize, cur_unreadable: bool, table_unreadable: bool, info: &mut StepInfo) -> Option<()> {
        let cur = self.cur.clone();
        let target = self.candidates(&cur, cur_unreadable).get(k - 1)?.clone();
        if target.heads.len() != cur.heads.len() {
            return None; // 树 ID 集合不同：今天的代码拒（TargetTreeIdentifiersDifferFromTheCurrentVersionWhoseHandlingIsUndecided）
        }
        if table_unreadable && config.table_fallback == TableFallback::Refuse {
            return None; // 今天的代码：CurrentInstanceTableMalformed
        }
        let ring: Vec<&Ver> = self
            .ring
            .iter()
            .flatten()
            .filter(|s| s.readable)
            .filter(|s| !(cur_unreadable && s.v.txg == cur.txg && s.v.inst == cur.inst))
            .map(|s| &s.v)
            .collect();
        let next = cur.txg + 1;
        let mut wms = Vec::new();
        for (h, ch) in cur.heads.iter().enumerate() {
            let all = ring.iter().filter(|v| v.heads.len() > h).map(|v| v.heads[h].wm).max();
            let kept_by_table = ring.iter().filter(|v| v.heads.len() > h && !abandoned_by(&cur.table, v.inst, v.txg)).map(|v| v.heads[h].wm).max();
            let kept = if table_unreadable {
                match config.table_fallback {
                    TableFallback::AllKept => all,
                    TableFallback::CurOnly | TableFallback::Refuse => None,
                }
            } else {
                kept_by_table
            };
            if h == 0 {
                let a = all.unwrap_or(0);
                let kb = kept_by_table.unwrap_or(0);
                info.jump_from_abandoned_only = a > ch.wm && kb <= ch.wm;
                info.kept_ring_above_cur = kb > ch.wm;
            }
            let wm = match config.wm_rule {
                WmRule::MaxCurAllRing => all.map_or(ch.wm, |r| r.max(ch.wm)),
                WmRule::MaxCurKeptRing => kept.map_or(ch.wm, |r| r.max(ch.wm)),
                WmRule::MaxAllRingOnly => all.unwrap_or(ch.wm),
                WmRule::MaxKeptRingOnly => kept.unwrap_or(ch.wm),
                WmRule::Target => target.heads[h].wm,
                WmRule::TargetHead0Only => {
                    if h == 0 {
                        target.heads[h].wm
                    } else {
                        all.map_or(ch.wm, |r| r.max(ch.wm))
                    }
                }
            };
            if wm < ch.wm {
                info.rollback_below_cur = true;
            }
            wms.push(wm);
        }
        let lin = Lin {
            t_rb: next,
            t_old: target.txg,
            t_cur: cur.txg,
            h_cur: cur.heads.iter().map(|h| h.wm).collect(),
            count: cur.heads.iter().map(|h| u64::try_from(h.log.len()).expect("小")).collect(),
            h_new: wms.clone(),
        };
        let mut v = cur.clone();
        v.txg = next;
        v.jsn = cur.jsn + 1;
        for (h, head) in v.heads.iter_mut().enumerate() {
            if wms[h] != head.wm {
                head.rows_all.push((next, wms[h]));
            }
            head.issued = union_sorted(&head.issued, &target.heads[h].tree);
            head.tree = target.heads[h].tree.clone();
            head.wm = wms[h];
            update_c2i(&mut v.c2i, cur.inst, h, wms[h], next);
        }
        v.lineage.push(lin);
        info.rollback = true;
        self.publish(v);
        Some(())
    }

    fn hide(&mut self, also_record: bool) -> Option<(usize, Option<usize>)> {
        let newest = self.newest_readable()?.clone();
        if newest.txg != self.cur.txg || newest.inst != self.cur.inst {
            return None;
        }
        let slot = usize::try_from(newest.txg).expect("小") % self.ring.len();
        self.ring[slot].as_mut().expect("在环里").readable = false;
        let mut rec_slot = None;
        if also_record {
            let j = usize::try_from(newest.jsn).expect("小") % self.journal.len();
            if let Some(r) = self.journal[j].as_mut() {
                r.readable = false;
                rec_slot = Some(j);
            }
        }
        Some((slot, rec_slot))
    }

    fn unhide(&mut self, slot: usize, rec_slot: Option<usize>) {
        if let Some(s) = self.ring[slot].as_mut() {
            s.readable = true;
        }
        if let Some(j) = rec_slot {
            if let Some(r) = self.journal[j].as_mut() {
                r.readable = true;
            }
        }
    }

    fn note_creation(&self, info: &mut StepInfo, h: usize, pair: Pair) {
        info.issued_now.push((h, pair));
        if self.ever.contains(&(h, pair.0, pair.1)) {
            info.first_txg_collides = true;
        }
    }

    fn apply(&mut self, config: &Config, op: Op) -> Option<StepInfo> {
        let cur = self.cur.clone();
        let mut info = StepInfo::default();
        match op {
            Op::Create(h) => {
                let h = usize::from(h);
                if h >= cur.heads.len() {
                    return None;
                }
                let (changes, pair) = create_change(&cur, h, cur.txg + 1);
                self.note_creation(&mut info, h, pair);
                let v = State::derive(&cur, cur.inst, cur.txg + 1, cur.jsn + 1, cur.floor, cur.table.clone(), changes);
                self.publish(v);
            }
            Op::Delete => {
                let victim = cur.heads[0].tree.iter().copied().filter(|(n, _)| *n >= 2).max()?;
                let mut changes = same_heads(&cur);
                changes[0].tree.retain(|p| *p != victim);
                let mut v = State::derive(&cur, cur.inst, cur.txg + 1, cur.jsn + 1, cur.floor, cur.table.clone(), changes);
                v.tombs.push((victim.0, victim.1, cur.txg + 1));
                self.publish(v);
            }
            Op::Rollback(k) => self.rollback(config, usize::from(k), false, false, &mut info)?,
            Op::RollbackCurUnreadable => self.rollback(config, 1, true, false, &mut info)?,
            Op::RollbackTableUnreadable => self.rollback(config, 1, false, true, &mut info)?,
            Op::RaiseFloor => {
                if cur.floor >= cur.txg {
                    return None;
                }
                let v = State::derive(&cur, cur.inst, cur.txg + 1, cur.jsn + 1, cur.txg, cur.table.clone(), same_heads(&cur));
                self.publish(v);
            }
            Op::Empty => {
                let v = State::derive(&cur, cur.inst, cur.txg + 1, cur.jsn + 1, cur.floor, cur.table.clone(), same_heads(&cur));
                self.publish(v);
            }
            Op::Mount | Op::CrashMidCreate => {
                let selected = self.newest_readable()?.clone();
                self.recover_onto(config, &selected);
                info.swept = self.sweep(config);
            }
            Op::CrashHide | Op::CrashHideRootAndRecord => {
                let (slot, rec_slot) = self.hide(op == Op::CrashHideRootAndRecord)?;
                let selected = self.newest_readable()?.clone();
                self.recover_onto(config, &selected);
                self.unhide(slot, rec_slot);
                info.swept = self.sweep(config);
            }
            Op::SwitchRedo | Op::SwitchError => {
                let selected = self.newest_readable()?.clone();
                let new_instance = self.newest_instance + 1;
                self.newest_instance = new_instance;
                let table = rows_for_a_new_instance(&selected, new_instance);
                let txg = self.first_txg(config, &selected);
                let jsn = self.highest_readable_jsn() + 1;
                info.switched = true;
                let changes = if op == Op::SwitchRedo {
                    let (changes, pair) = create_change(&selected, 0, txg);
                    self.note_creation(&mut info, 0, pair);
                    changes
                } else {
                    let mut changes = same_heads(&selected);
                    if config.recovery_wm == RecoveryWm::SelectedMinusOneNotBelowTwoAlsoSwitchError {
                        changes[0].wm = changes[0].wm.saturating_sub(1).max(2);
                    }
                    changes
                };
                let v = State::derive(&selected, new_instance, txg, jsn, selected.floor, table, changes);
                self.publish(v);
            }
            Op::Clone => {
                if cur.heads.len() != 1 {
                    return None;
                }
                let snapshot = match config.clone_from {
                    CloneFrom::Current => cur.clone(),
                    CloneFrom::OlderVersion => self.candidates(&cur, false).first()?.clone(),
                };
                let txg = cur.txg + 1;
                let s = &snapshot.heads[0];
                let wm = match config.clone_wm {
                    CloneWm::Snapshot => s.wm,
                    CloneWm::OriginCurrent => cur.heads[0].wm,
                };
                let mut rows_all = s.rows_all.clone();
                if config.clone_writes_a_row && wm != s.wm {
                    rows_all.push((txg, wm));
                }
                let mut inherited_lineage: Vec<(u64, u64)> = s.inherited_lineage.clone();
                inherited_lineage.extend(snapshot.lineage.iter().map(|l| (l.t_rb, l.h_cur[0])));
                let mut inherited_c2i: Vec<(u64, u64)> = s.inherited_c2i.clone();
                inherited_c2i.extend(snapshot.c2i.iter().filter(|e| e.1 == 0).map(|e| (e.2, e.3)));
                let b = Head {
                    wm,
                    tree: s.tree.clone(),
                    rows_all,
                    rows_create: s.rows_create.clone(),
                    log: s.log.clone(),
                    issued: s.issued.clone(),
                    inherited_lineage,
                    inherited_c2i,
                };
                let mut v = State::derive(&cur, cur.inst, txg, cur.jsn + 1, cur.floor, cur.table.clone(), same_heads(&cur));
                update_c2i(&mut v.c2i, cur.inst, 1, wm, txg);
                v.heads.push(b);
                self.publish(v);
            }
        }
        Some(info)
    }
}

fn two_births(pairs: &[Pair]) -> bool {
    pairs.windows(2).any(|w| w[0].0 == w[1].0 && w[0].1 != w[1].1)
}
fn floor_red(rows: &[(u64, u64)], wm: u64) -> bool {
    rows.iter().any(|&(_, h)| h > wm)
}
fn cut_red(rows: &[(u64, u64)], tree: &[Pair]) -> bool {
    tree.iter().any(|&(n, b)| rows.iter().any(|&(g, h)| h > n && b > g))
}
fn cover_red(rows: &[(u64, u64)], wm: u64, txg: u64) -> bool {
    wm > 2 && !rows.iter().any(|&(g, h)| h >= wm && g <= txg)
}
fn max_h_upto(rows: &[(u64, u64)], t: u64) -> u64 {
    rows.iter().filter(|(g, _)| *g <= t).map(|(_, h)| *h).max().unwrap_or(0)
}
fn log_upto(log: &[Pair], t: u64) -> u64 {
    u64::try_from(log.iter().filter(|(_, b)| *b <= t).count()).expect("小")
}

/// 死亡 txg 在不在这一版的谱系父链上（I-9.18 的定义：有谱系记录 (T_rb, T_old) 的 txg 的父是 T_old，其余是 txg − 1）。
fn on_the_lineage(v: &Ver, death: u64) -> bool {
    let mut txg = v.txg;
    loop {
        if txg == death {
            return true;
        }
        if txg < death || txg <= 1 {
            return false;
        }
        txg = v.lineage.iter().find(|l| l.t_rb == txg).map_or(txg - 1, |l| l.t_old);
    }
}

const CANDIDATES: [&str; 26] = [
    "I-9.29",
    "I-9.6",
    "I-9.29-pooled",
    "C2aL",
    "C3rL",
    "C3rL-pooled",
    "C4g",
    "C4g-maxW",
    "C4g-firstW",
    "C2i",
    "C2i-inst",
    "C2ig",
    "C2a-create-strict",
    "C2a-create-tolB",
    "C2a-create-tolC",
    "C3r-cover-strict",
    "C3r-cover-tolB",
    "C3r-cover-tolC",
    "pair(Tcur,Hnew)",
    "C2a-create-12",
    "C3r-nocover",
    "I-9.29t",
    "I-9.6t",
    "C4g-inh",
    "C2i-inh",
    "C2ig-inh",
];
const N: usize = CANDIDATES.len();

/// 判一张镜像（最新那条可读的根）：每个候选红不红。
fn verdicts(state: &State) -> [bool; N] {
    let v = state.newest_readable().expect("至少一条");
    let ring = state.readable();
    let cand: Vec<&Ver> = ring.iter().copied().filter(|o| !abandoned_by(&v.table, o.inst, o.txg) && o.txg >= v.floor).collect();
    let abandoned: Vec<&Ver> = ring.iter().copied().filter(|o| abandoned_by(&v.table, o.inst, o.txg)).collect();
    let mut r = [false; N];
    let mut pooled_pairs: Vec<Pair> = Vec::new();
    let mut pooled_log: Vec<Pair> = Vec::new();
    // C2i-inst：每实例取各头里最大的那一条。
    let mut per_inst: BTreeMap<u32, (u64, u64)> = BTreeMap::new();
    for &(i, _, m, g) in &v.c2i {
        let e = per_inst.entry(i).or_insert((m, g));
        if m > e.0 {
            *e = (m, g);
        }
    }
    for (h, hd) in v.heads.iter().enumerate() {
        let (wm, tree) = (hd.wm, &hd.tree);
        let mut pairs: Vec<Pair> = Vec::new();
        let mut largest = 0;
        for o in &cand {
            if o.heads.len() > h {
                pairs.extend(o.heads[h].tree.iter().copied());
                largest = largest.max(o.heads[h].tree.iter().map(|p| p.0).max().unwrap_or(0));
            }
        }
        pairs.sort_unstable();
        pairs.dedup();
        pooled_pairs.extend(pairs.iter().copied());
        pooled_log.extend(hd.log.iter().copied());
        r[0] |= two_births(&pairs);
        r[1] |= wm <= largest;
        // C2aL：I-9.30 ①②③ 配每次水位变了的台阶行，加谱系那一对判等于。
        let pair_eq = v.lineage.iter().any(|l| h < l.h_cur.len() && (l.h_cur[h] == 0 || max_h_upto(&hd.rows_all, l.t_cur) != l.h_cur[h]));
        let c2al = floor_red(&hd.rows_all, wm) || cut_red(&hd.rows_all, tree) || cover_red(&hd.rows_all, wm, v.txg) || pair_eq;
        r[3] |= c2al;
        // C3rL：C3rs 加谱系带 |log(cur)| 判等于。
        let log_n_over = hd.log.iter().any(|(n, _)| *n >= wm);
        let c3rs = two_births(&hd.log) || log_n_over || tree.iter().any(|p| hd.log.binary_search(p).is_err());
        let count_eq = v.lineage.iter().any(|l| h < l.count.len() && (l.count[h] == 0 || log_upto(&hd.log, l.t_cur) != l.count[h]));
        r[4] |= c3rs || count_eq;
        // C4g 按头：(T_rb, W_h)。
        let c4g = v.lineage.iter().any(|l| {
            h < l.h_cur.len() && {
                let w = l.h_cur[h];
                wm < w || tree.iter().any(|&(n, b)| n < w && b > l.t_rb)
            }
        });
        r[6] |= c4g;
        let c4g_with = |w: u64, l: &Lin| wm < w || tree.iter().any(|&(n, b)| n < w && b > l.t_rb);
        r[7] |= v.lineage.iter().any(|l| h < l.h_cur.len() && c4g_with(*l.h_cur.iter().max().expect("至少一个头"), l));
        r[8] |= v.lineage.iter().any(|l| h < l.h_cur.len() && c4g_with(l.h_cur[0], l));
        // C2i 按 (实例, 头)。
        let c2i_with = |m: u64, g: u64| wm < m || tree.iter().any(|&(n, b)| n < m && b > g);
        let c2i = v.c2i.iter().any(|&(_, eh, m, g)| eh == h && c2i_with(m, g));
        r[9] |= c2i;
        r[10] |= per_inst.values().any(|&(m, g)| c2i_with(m, g));
        r[11] |= c2i || c4g;
        // 弱读法与容许跳升的两种读法。
        let tol_b = abandoned.iter().any(|o| o.heads.len() > h && o.heads[h].wm >= wm);
        let tol_c = v.lineage.iter().any(|l| h < l.h_new.len() && l.h_new[h] >= wm);
        let c2a12 = floor_red(&hd.rows_create, wm) || cut_red(&hd.rows_create, tree);
        let c2a3 = cover_red(&hd.rows_create, wm, v.txg);
        r[12] |= c2a12 || c2a3;
        r[13] |= c2a12 || (c2a3 && !tol_b);
        r[14] |= c2a12 || (c2a3 && !tol_c);
        let c3r = two_births(&hd.log) || log_n_over;
        let log_max = hd.log.iter().map(|(n, _)| *n).max().unwrap_or(0);
        let c3r_cover = wm > 2 && log_max + 1 < wm;
        r[15] |= c3r || c3r_cover;
        r[16] |= c3r || (c3r_cover && !tol_b);
        r[17] |= c3r || (c3r_cover && !tol_c);
        r[18] |= v.lineage.iter().any(|l| h < l.h_new.len() && max_h_upto(&hd.rows_all, l.t_cur) < l.h_new[h]);
        r[19] |= c2a12;
        r[20] |= c3r;
        // 第二轮辩方的「现有检查」口径：I-9.29 加谱系上的墓碑，I-9.6 加墓碑那一半（墓碑只在头 0）。
        let mut pairs_t: Vec<Pair> = Vec::new();
        for o in &cand {
            if o.heads.len() > h {
                pairs_t.extend(o.heads[h].tree.iter().copied());
                if h == 0 {
                    pairs_t.extend(o.tombs.iter().filter(|t| on_the_lineage(o, t.2)).map(|t| (t.0, t.1)));
                }
            }
        }
        pairs_t.sort_unstable();
        pairs_t.dedup();
        r[21] |= two_births(&pairs_t);
        let c4g_inh = c4g || hd.inherited_lineage.iter().any(|&(t_rb, w)| wm < w || tree.iter().any(|&(n, b)| n < w && b > t_rb));
        let c2i_inh = c2i || hd.inherited_c2i.iter().any(|&(m, g)| c2i_with(m, g));
        r[23] |= c4g_inh;
        r[24] |= c2i_inh;
        r[25] |= c4g_inh || c2i_inh;
        r[22] |= wm <= largest || (h == 0 && v.tombs.iter().any(|t| t.0 >= wm));
    }
    pooled_pairs.sort_unstable();
    pooled_pairs.dedup();
    pooled_log.sort_unstable();
    pooled_log.dedup();
    r[2] = r[0] || two_births(&pooled_pairs);
    r[5] = r[4] || two_births(&pooled_log);
    r
}

const KINDS: [&str; 6] = ["false_positive", "event_missed", "residual_missed", "hole_event_missed", "hole_residual_missed", "red_on_collision"];

#[derive(Default)]
struct Tally {
    full_length_histories: u64,
    images: u64,
    legal_images: u64,
    events: u64,
    residual_images: u64,
    hole_events: u64,
    hole_residual_images: u64,
    reissues_across_abandoned_roots: u64,
    rollbacks: u64,
    jumps_from_abandoned_only: u64,
    kept_ring_above_cur: u64,
    rollback_below_cur: u64,
    sweeps: u64,
    switches: u64,
    collisions: u64,
    collision_images_with_cross_instance_txg_not_increasing: u64,
    images_with_cross_instance_txg_not_increasing: u64,
    counts: Vec<[u64; 6]>,
    examples: Vec<Vec<Option<Vec<u8>>>>,
}

fn better(candidate: &[u8], kept: &Option<Vec<u8>>) -> bool {
    match kept {
        None => true,
        Some(k) => (candidate.len(), candidate) < (k.len(), k.as_slice()),
    }
}

/// journal 里读得出的记录按 jsn 排，相邻两条换到更大的实例代号而 txg 不增（I-8.12 今天不判的那一格）。
fn cross_instance_txg_not_increasing(state: &State) -> bool {
    let mut recs: Vec<Rec> = state.journal.iter().flatten().filter(|r| r.readable).copied().collect();
    recs.sort_by_key(|r| r.jsn);
    recs.windows(2).any(|w| w[1].inst > w[0].inst && w[1].txg <= w[0].txg)
}

impl Tally {
    fn new() -> Tally {
        Tally { counts: vec![[0; 6]; N], examples: vec![vec![None; KINDS.len()]; N], ..Tally::default() }
    }
    fn hit(&mut self, c: usize, kind: usize, seq: &[u8]) {
        self.counts[c][kind] += 1;
        if better(seq, &self.examples[c][kind]) {
            self.examples[c][kind] = Some(seq.to_vec());
        }
    }
    fn judge(&mut self, state: &State, step: &StepInfo, seq: &[u8]) {
        self.images += 1;
        if step.rollback {
            self.rollbacks += 1;
            self.jumps_from_abandoned_only += u64::from(step.jump_from_abandoned_only);
            self.kept_ring_above_cur += u64::from(step.kept_ring_above_cur);
            self.rollback_below_cur += u64::from(step.rollback_below_cur);
        }
        self.sweeps += u64::from(step.swept);
        self.switches += u64::from(step.switched);
        let red = verdicts(state);
        let v = state.newest_readable().expect("至少一条");
        let legal = v.heads.iter().all(|h| !two_births(&h.issued) && !h.issued.iter().any(|(n, _)| *n >= h.wm));
        let event = step.issued_now.iter().any(|&(h, (n, b))| v.heads[h].issued.iter().any(|&(o, ob)| o == n && ob != b));
        let residual = !event && v.heads.iter().any(|h| h.tree.iter().any(|&(n, b)| h.issued.iter().any(|&(o, ob)| o == n && ob < b)));
        let existing_silent = !red[21] && !red[22];
        let across = step.issued_now.iter().any(|&(h, (n, b))| {
            state.readable().iter().any(|o| abandoned_by(&v.table, o.inst, o.txg) && o.heads.len() > h && o.heads[h].tree.iter().any(|&(on, ob)| on == n && ob != b))
        });
        if across && !event {
            self.reissues_across_abandoned_roots += 1;
        }
        let xi = cross_instance_txg_not_increasing(state);
        self.images_with_cross_instance_txg_not_increasing += u64::from(xi);
        if step.first_txg_collides {
            self.collisions += 1;
            self.collision_images_with_cross_instance_txg_not_increasing += u64::from(xi);
        }
        self.legal_images += u64::from(legal);
        self.events += u64::from(event);
        self.residual_images += u64::from(residual);
        self.hole_events += u64::from(event && existing_silent);
        self.hole_residual_images += u64::from(residual && existing_silent);
        for (c, &is_red) in red.iter().enumerate() {
            if legal && is_red {
                self.hit(c, 0, seq);
            }
            if event && !is_red {
                self.hit(c, 1, seq);
                if existing_silent {
                    self.hit(c, 3, seq);
                }
            }
            if residual && !is_red {
                self.hit(c, 2, seq);
                if existing_silent {
                    self.hit(c, 4, seq);
                }
            }
            if step.first_txg_collides && is_red {
                self.hit(c, 5, seq);
            }
        }
    }
}

fn explore(config: &Config, state: &State, seq: &mut Vec<u8>, tally: &mut Tally) {
    if seq.len() == config.longest {
        tally.full_length_histories += 1;
        return;
    }
    for (index, op) in config.ops.iter().enumerate() {
        let mut next = state.clone();
        let Some(step) = next.apply(config, *op) else { continue };
        seq.push(u8::try_from(index).expect("字母少"));
        tally.judge(&next, &step, seq);
        explore(config, &next, seq, tally);
        seq.pop();
    }
}

fn render(state: &State) -> String {
    let v = state.newest_readable().expect("至少一条");
    let p = |l: &[Pair]| l.iter().map(|(a, b)| format!("{a}@{b}")).collect::<Vec<_>>().join(",");
    let r = |l: &[(u64, u64)]| l.iter().map(|(a, b)| format!("{a}:{b}")).collect::<Vec<_>>().join(",");
    let heads: Vec<String> = v
        .heads
        .iter()
        .enumerate()
        .map(|(h, hd)| format!("h{h}{{wm={} tree=[{}] rows_all=[{}] rows_create=[{}] log=[{}] issued=[{}]}}", hd.wm, p(&hd.tree), r(&hd.rows_all), r(&hd.rows_create), p(&hd.log), p(&hd.issued)))
        .collect();
    let lin: Vec<String> = v
        .lineage
        .iter()
        .map(|l| format!("{}<-{}|cur{}:H{:?}:n{:?}:Hnew{:?}", l.t_rb, l.t_old, l.t_cur, l.h_cur, l.count, l.h_new))
        .collect();
    let ring: Vec<String> = state
        .ring
        .iter()
        .flatten()
        .map(|s| format!("{}:{}:wm{:?}{}", s.v.inst, s.v.txg, s.v.heads.iter().map(|h| h.wm).collect::<Vec<_>>(), if s.readable { "" } else { "(hidden)" }))
        .collect();
    let table: Vec<String> = v.table.iter().map(|(i, t)| format!("{i}:{t}")).collect();
    let c2i: Vec<String> = v.c2i.iter().map(|(i, h, m, g)| format!("{i}/{h}:{m}@{g}")).collect();
    let mut recs: Vec<Rec> = state.journal.iter().flatten().copied().collect();
    recs.sort_by_key(|x| x.jsn);
    let journal: Vec<String> = recs.iter().map(|x| format!("{}:{}:{}{}", x.jsn, x.inst, x.txg, if x.readable { "" } else { "(hidden)" })).collect();
    format!(
        "newest={}:{} F={} table=[{}] {} lineage=[{}] c2i=[{}] ring=[{}] journal=[{}]",
        v.inst, v.txg, v.floor, table.join(","), heads.join(" "), lin.join(","), c2i.join(","), ring.join(","), journal.join(",")
    )
}

fn trace(config: &Config, seq: &[u8]) -> Vec<String> {
    let mut state = State::initial(config);
    let mut lines = vec![format!("step=start {}", render(&state))];
    for (i, index) in seq.iter().enumerate() {
        let op = config.ops[usize::from(*index)];
        let _ = state.apply(config, op).expect("复放");
        let red = verdicts(&state);
        let names: Vec<&str> = CANDIDATES.iter().zip(red).filter(|(_, r)| *r).map(|(n, _)| *n).collect();
        lines.push(format!("step={i} op={} {} red=[{}]", op.code(), render(&state), names.join(",")));
    }
    lines
}

fn label(c: &Config) -> String {
    let t = if c.sweep_threshold == u64::MAX { "never".to_string() } else { c.sweep_threshold.to_string() };
    let ops: Vec<String> = c.ops.iter().map(|o| o.code()).collect();
    format!(
        "group={} rows_create_stop_after_a_rollback={} wm_rule={:?} table_fallback={:?} recovery_wm={:?} first_txg={:?} clone={:?}/{:?}/row={} ring={} journal={} sweep_threshold={t} longest={} ops={}",
        c.group,
        c.rows_create_stop_after_a_rollback,
        c.wm_rule,
        c.table_fallback,
        c.recovery_wm,
        c.first_txg,
        c.clone_from,
        c.clone_wm,
        c.clone_writes_a_row,
        c.ring,
        c.journal,
        c.longest,
        ops.join(",")
    )
}

fn run(config: &Config) {
    ROWS_CREATE_STOP_AFTER_A_ROLLBACK.store(config.rows_create_stop_after_a_rollback, std::sync::atomic::Ordering::Relaxed);
    let mut tally = Tally::new();
    explore(config, &State::initial(config), &mut Vec::new(), &mut tally);
    let l = label(config);
    let t = &tally;
    println!(
        "W3R3MODEL {l} full_length_histories={} images={} legal_images={} events={} residual_images={} hole_events={} hole_residual_images={} reissues_across_abandoned_roots={} rollbacks={} rollback_jumps_from_abandoned_roots_only={} rollback_kept_ring_above_cur={} rollback_below_cur={} sweeps={} switches={} identity_collisions={} collision_images_with_cross_instance_txg_not_increasing={} images_with_cross_instance_txg_not_increasing={}",
        t.full_length_histories,
        t.images,
        t.legal_images,
        t.events,
        t.residual_images,
        t.hole_events,
        t.hole_residual_images,
        t.reissues_across_abandoned_roots,
        t.rollbacks,
        t.jumps_from_abandoned_only,
        t.kept_ring_above_cur,
        t.rollback_below_cur,
        t.sweeps,
        t.switches,
        t.collisions,
        t.collision_images_with_cross_instance_txg_not_increasing,
        t.images_with_cross_instance_txg_not_increasing
    );
    for (c, name) in CANDIDATES.iter().enumerate() {
        let counts: Vec<String> = KINDS.iter().enumerate().map(|(k, kn)| format!("{kn}={}", t.counts[c][k])).collect();
        println!("W3R3COUNT {l} candidate={name} {}", counts.join(" "));
    }
    for (c, name) in CANDIDATES.iter().enumerate() {
        for (k, kn) in KINDS.iter().enumerate() {
            if let Some(seq) = &t.examples[c][k] {
                let ops: Vec<String> = seq.iter().map(|i| config.ops[usize::from(*i)].code()).collect();
                println!("W3R3EXAMPLE {l} candidate={name} kind={kn} ops=[{}]", ops.join(","));
                if std::env::var("W3R3_TRACE").is_ok() {
                    for line in trace(config, seq) {
                        println!("W3R3TRACE candidate={name} kind={kn} {line}");
                    }
                }
            }
        }
    }
}

fn configs() -> Vec<Config> {
    let base = Config {
        group: "verify",
        rows_create_stop_after_a_rollback: false,
        wm_rule: WmRule::MaxCurAllRing,
        table_fallback: TableFallback::Refuse,
        recovery_wm: RecoveryWm::Selected,
        first_txg: FirstTxg::Correct,
        clone_from: CloneFrom::Current,
        clone_wm: CloneWm::Snapshot,
        clone_writes_a_row: true,
        ring: 4,
        journal: 8,
        sweep_threshold: u64::MAX,
        longest: 7,
        ops: R2_OPS.to_vec(),
    };
    let mut out = Vec::new();
    // verify：独立重算前两轮的数（环 4、长 7、十二个字母）。
    out.push(Config { ..base.clone() });
    out.push(Config { sweep_threshold: 0, ..base.clone() });
    out.push(Config { wm_rule: WmRule::Target, ..base.clone() });
    out.push(Config { recovery_wm: RecoveryWm::SelectedMinusOne, ..base.clone() });
    out.push(Config { recovery_wm: RecoveryWm::SelectedMinusOneNotBelowTwoAlsoSwitchError, ..base.clone() });
    // y1：Y1 第二种取法在同一组历史上。
    out.push(Config { group: "y1", wm_rule: WmRule::MaxCurKeptRing, ..base.clone() });
    out.push(Config { group: "y1", wm_rule: WmRule::MaxCurKeptRing, sweep_threshold: 0, ..base.clone() });
    // y1x：加回退那一刻 cur 的根读不出、实例表读不出、根与记录都读不出三个字母。
    let y1x_ops = vec![
        Op::Create(0),
        Op::Delete,
        Op::Rollback(1),
        Op::Rollback(2),
        Op::Empty,
        Op::Mount,
        Op::CrashHide,
        Op::RollbackCurUnreadable,
        Op::RollbackTableUnreadable,
        Op::CrashHideRootAndRecord,
    ];
    let y1x = Config { group: "y1x", ops: y1x_ops, ..base.clone() };
    for (wm_rule, fallback) in [
        (WmRule::MaxCurAllRing, TableFallback::Refuse),
        (WmRule::MaxCurKeptRing, TableFallback::Refuse),
        (WmRule::MaxCurKeptRing, TableFallback::AllKept),
        (WmRule::MaxCurKeptRing, TableFallback::CurOnly),
        (WmRule::MaxAllRingOnly, TableFallback::Refuse),
        (WmRule::MaxKeptRingOnly, TableFallback::Refuse),
        (WmRule::Target, TableFallback::Refuse),
    ] {
        out.push(Config { wm_rule, table_fallback: fallback, ..y1x.clone() });
    }
    // y1m：字面 C2a 的台阶行在回退之后不写，配 V1（回退取目标）与产品取法，看「容许跳升」的两种读法放不放过重发。
    for wm_rule in [WmRule::Target, WmRule::MaxCurAllRing, WmRule::MaxAllRingOnly] {
        out.push(Config { group: "y1m", rows_create_stop_after_a_rollback: true, wm_rule, ..y1x.clone() });
    }
    // clone：两个可写头。
    let clone_ops = vec![Op::Create(0), Op::Create(1), Op::Clone, Op::Rollback(1), Op::Rollback(2), Op::Empty, Op::Mount, Op::CrashHide, Op::Delete];
    let cl = Config { group: "clone", ops: clone_ops, ..base.clone() };
    for (wm_rule, from, cwm, row) in [
        (WmRule::MaxCurAllRing, CloneFrom::Current, CloneWm::Snapshot, true),
        (WmRule::MaxCurAllRing, CloneFrom::OlderVersion, CloneWm::Snapshot, true),
        (WmRule::MaxCurAllRing, CloneFrom::OlderVersion, CloneWm::OriginCurrent, true),
        (WmRule::MaxCurAllRing, CloneFrom::OlderVersion, CloneWm::OriginCurrent, false),
        (WmRule::MaxCurKeptRing, CloneFrom::OlderVersion, CloneWm::Snapshot, true),
        (WmRule::Target, CloneFrom::OlderVersion, CloneWm::Snapshot, true),
        (WmRule::TargetHead0Only, CloneFrom::OlderVersion, CloneWm::Snapshot, true),
        (WmRule::TargetHead0Only, CloneFrom::Current, CloneWm::Snapshot, true),
    ] {
        out.push(Config { wm_rule, clone_from: from, clone_wm: cwm, clone_writes_a_row: row, ..cl.clone() });
    }
    // first：新实例第一个 txg 改坏，与被抛弃那次发布的出生代撞。
    let first_ops = vec![
        Op::Create(0),
        Op::Delete,
        Op::Rollback(1),
        Op::Empty,
        Op::Mount,
        Op::CrashHide,
        Op::CrashHideRootAndRecord,
        Op::SwitchRedo,
        Op::SwitchError,
    ];
    let fi = Config { group: "first", ops: first_ops, ..base.clone() };
    for first_txg in [FirstTxg::Correct, FirstTxg::SelectedPlusOne] {
        out.push(Config { first_txg, ..fi.clone() });
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let group = args.first().map_or("all", String::as_str).to_string();
    let only: Option<usize> = args.get(1).map(|s| s.parse().expect("组内序号"));
    let longest: Option<usize> = std::env::var("W3R3_LONGEST").ok().map(|s| s.parse().expect("长"));
    let mut index_in_group: BTreeMap<&str, usize> = BTreeMap::new();
    for mut config in configs() {
        let index = index_in_group.entry(config.group).or_insert(0);
        let this = *index;
        *index += 1;
        if (group == "all" || group == config.group) && only.is_none_or(|w| w == this) {
            if let Some(l) = longest {
                config.longest = l;
            }
            run(&config);
        }
    }
}
