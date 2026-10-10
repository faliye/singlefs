//! m3-w3-r3 攻方腿第二次抽样（Opus s2）的抽象时间线模型，从头写，不依赖 crates/、不拷前两轮的模型代码。
//! 条文走法（推的，照 D23 已定项 14、D18 已定项 11、D8 已定项 6 读）：
//! - 普通发布 txg = cur 的 txg + 1；建 inode 取所在头的水位、水位加一；删取那个头里号最大的（号 ≥ 2）。
//! - 管理员回退：候选集 = 环里读得出、按 cur 的实例表不被抛弃、头集合与 cur 相同（今天的 mount.rs 头集合不同就拒）的根；
//!   新根各头的树取目标那一版的，水位按 Y1 取法：Opt1 = max(cur 内存里的, 环里读得出的全部根)；Opt2 = max(cur 内存里的, 环里读得出、没被 cur 的表抛弃的根)。
//!   阳性对照臂：V1 取目标那一版的；VR 只取环里读得出的根（C143 收严之前的式子，配「回退那一刻 cur 的根读不出」那一步）；V3 恢复时头 1 的水位少算 1。
//! - 恢复（挂载、崩溃）：新实例接在所选那一版（环里读得出的最新那条根）后面写行那次发布；第一个 txg 按 FirstTxg 取
//!   （Product = max(读得出的根, 读得出的记录) + 1，mount.rs 的 first_txg_of_new_instance；两种变异）；行照 [max(所选实例,1), 新实例) 写。
//! - 历史（各候选的盘上东西）随这一版走：发布从上一版带，回退从 cur 带并追加谱系，恢复从所选那一版带。
//! 由用户定的步全部穷举，只固定起点（新池新建文件之后：实例 1、txg 3、水位 2、树 [(1,3)]）。
#![allow(clippy::too_many_lines)]
use std::collections::BTreeMap;

type Obj = (u32, u64, u64); // (树, 号, 出生代)

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WmRule { Opt1, Opt2 }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm { Product, V1, VR, V3 }
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FirstTxg { Product, RingRootsOnly, SelectedPlusOne }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Op {
    Create(u32),
    Delete(u32),
    Rollback(u8),
    RollbackCurHidden(u8),
    Empty,
    Mount,
    Clone,
    CrashHide,
    CrashHideRR,
    CrashHide2,
    CrashMidCreate,
}

impl Op {
    fn code(self) -> String {
        match self {
            Op::Create(t) => format!("create{t}"),
            Op::Delete(t) => format!("delete{t}"),
            Op::Rollback(k) => format!("rollback{k}"),
            Op::RollbackCurHidden(k) => format!("rollback{k}-cur-hidden"),
            Op::Empty => "empty".into(),
            Op::Mount => "mount".into(),
            Op::Clone => "clone".into(),
            Op::CrashHide => "crash-hide".into(),
            Op::CrashHideRR => "crash-hide-root-and-record".into(),
            Op::CrashHide2 => "crash-hide-two".into(),
            Op::CrashMidCreate => "crash-mid-create".into(),
        }
    }
}

const K2_OPS: [Op; 9] = [Op::Create(1), Op::Delete(1), Op::Rollback(1), Op::Rollback(2), Op::RollbackCurHidden(1), Op::Empty, Op::Mount, Op::CrashHide, Op::CrashMidCreate];
const CLONE_OPS: [Op; 9] = [Op::Create(1), Op::Create(2), Op::Clone, Op::Delete(1), Op::Rollback(1), Op::Rollback(2), Op::Mount, Op::CrashHide, Op::Empty];
const FT_OPS: [Op; 8] = [Op::Create(1), Op::Empty, Op::Rollback(1), Op::Mount, Op::CrashHide, Op::CrashHideRR, Op::CrashHide2, Op::CrashMidCreate];

#[derive(Clone, Copy, Debug)]
struct Config { rule: WmRule, arm: Arm, ft: FirstTxg, ring: usize, depth: usize, ops: &'static [Op], group: &'static str }

#[derive(Clone, PartialEq, Eq, Debug)]
struct Head { tree: u32, wm: u64, objs: Vec<(u64, u64)> }

#[derive(Clone, PartialEq, Eq, Debug, Default)]
struct Lin {
    t_rb: u64,
    t_old: u64,
    h_cur: Vec<(u32, u64)>,
    h_new: Vec<(u32, u64)>,
    count: Vec<(u32, u64)>,
    count_nocopy: Vec<(u32, u64)>,
    count_pool: u64,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
struct Hist {
    rows_all: Vec<(u32, u64, u64)>,
    rows_create: Vec<(u32, u64, u64)>,
    log: Vec<Obj>,
    log_nocopy: Vec<Obj>,
    lin: Vec<Lin>,
    peak: Vec<(u32, u32, u64, u64)>,
    peak_pool: Vec<(u32, u64, u64)>,
}

#[derive(Clone, Debug)]
struct Version { inst: u32, txg: u64, heads: Vec<Head>, table: Vec<(u32, u64)>, hist: Hist, issued: Vec<Obj> }

impl Version {
    fn head(&self, tree: u32) -> Option<&Head> { self.heads.iter().find(|h| h.tree == tree) }
    fn trees(&self) -> Vec<u32> { self.heads.iter().map(|h| h.tree).collect() }
    fn wm(&self, tree: u32) -> u64 { self.head(tree).map_or(0, |h| h.wm) }
}

fn abandoned_by(table: &[(u32, u64)], inst: u32, txg: u64) -> bool { table.iter().any(|&(i, t)| i == inst && txg > t) }

fn ins<T: Ord + Copy>(v: &mut Vec<T>, x: T) { if let Err(p) = v.binary_search(&x) { v.insert(p, x); } }

/// 照写规则从 base 带出下一版的历史（正确写法）。
fn derive(base: &Version, inst: u32, txg: u64, heads: Vec<Head>, table: Vec<(u32, u64)>, created: &[Obj], extra_issued: &[Obj]) -> Version {
    let mut hist = base.hist.clone();
    for h in &heads {
        let before = base.head(h.tree).map(|b| b.wm);
        if before != Some(h.wm) { hist.rows_all.push((h.tree, txg, h.wm)); }
        if created.iter().any(|o| o.0 == h.tree) { hist.rows_create.push((h.tree, txg, h.wm)); }
        match hist.peak.iter_mut().find(|e| e.0 == inst && e.1 == h.tree) {
            Some(e) => { if h.wm > e.2 { e.2 = h.wm; e.3 = txg; } }
            None => hist.peak.push((inst, h.tree, h.wm, txg)),
        }
    }
    let pool = heads.iter().map(|h| h.wm).max().unwrap_or(0);
    match hist.peak_pool.iter_mut().find(|e| e.0 == inst) {
        Some(e) => { if pool > e.1 { e.1 = pool; e.2 = txg; } }
        None => hist.peak_pool.push((inst, pool, txg)),
    }
    for &o in created { ins(&mut hist.log, o); ins(&mut hist.log_nocopy, o); }
    let mut issued = base.issued.clone();
    for &o in created.iter().chain(extra_issued) { ins(&mut issued, o); }
    Version { inst, txg, heads, table, hist, issued }
}

#[derive(Clone, Debug)]
struct Slot { v: Version, readable: bool }
#[derive(Clone, Copy, Debug)]
struct Rec { inst: u32, txg: u64, readable: bool }

#[derive(Clone)]
struct State { ring: Vec<Option<Slot>>, recs: Vec<Rec>, newest_inst: u32, cur: Version, written: Vec<(Obj, u32)> }

#[derive(Default, Clone)]
struct Step {
    created: Vec<Obj>,
    rollback: bool,
    jump_from_abandoned_only: bool,
    jump_from_valid: bool,
    cloned_log_entries: u64,
}

impl State {
    fn initial(ring: usize) -> State {
        let obj = (1, 1, 3);
        let base = Version { inst: 1, txg: 0, heads: Vec::new(), table: Vec::new(), hist: Hist::default(), issued: Vec::new() };
        let v = derive(&base, 1, 3, vec![Head { tree: 1, wm: 2, objs: vec![(1, 3)] }], Vec::new(), &[obj], &[]);
        let mut s = State { ring: vec![None; ring], recs: Vec::new(), newest_inst: 1, cur: v.clone(), written: vec![(obj, 1)] };
        s.publish(v);
        s
    }
    fn publish(&mut self, v: Version) {
        let n = self.ring.len();
        let slot = usize::try_from(v.txg).expect("小") % n;
        self.recs.push(Rec { inst: v.inst, txg: v.txg, readable: true });
        self.ring[slot] = Some(Slot { v: v.clone(), readable: true });
        self.cur = v;
    }
    fn readable(&self) -> Vec<&Version> {
        let mut out: Vec<&Version> = self.ring.iter().flatten().filter(|s| s.readable).map(|s| &s.v).collect();
        out.sort_by_key(|v| (v.txg, v.inst));
        out
    }
    fn newest(&self) -> Option<&Version> { self.readable().into_iter().last() }
    fn slot_of(&self, v: &Version) -> Option<usize> {
        self.ring.iter().position(|s| s.as_ref().is_some_and(|s| s.v.txg == v.txg && s.v.inst == v.inst))
    }
    fn rows_for_new_instance(sel: &Version, new_inst: u32) -> Vec<(u32, u64)> {
        let mut t = sel.table.clone();
        for i in sel.inst.max(1)..new_inst {
            t.retain(|&(x, _)| x != i);
            t.push((i, if i == sel.inst { sel.txg } else { 0 }));
        }
        t.sort_unstable();
        t
    }
    /// 恢复：在当前的可读性下（被藏的根与记录已标好）择根、算第一个 txg、发写行那次发布。
    fn recover(&mut self, cfg: &Config) -> Option<()> {
        let sel = self.newest()?.clone();
        let first = match cfg.ft {
            FirstTxg::Product => {
                let r = self.readable().iter().map(|v| v.txg).max().unwrap_or(0);
                let j = self.recs.iter().filter(|r| r.readable).map(|r| r.txg).max().unwrap_or(0);
                r.max(j) + 1
            }
            FirstTxg::RingRootsOnly => self.readable().iter().map(|v| v.txg).max().unwrap_or(0) + 1,
            FirstTxg::SelectedPlusOne => sel.txg + 1,
        };
        let new_inst = self.newest_inst + 1;
        self.newest_inst = new_inst;
        let table = State::rows_for_new_instance(&sel, new_inst);
        let mut heads = sel.heads.clone();
        if cfg.arm == Arm::V3 {
            if let Some(h) = heads.iter_mut().find(|h| h.tree == 1) { h.wm = h.wm.saturating_sub(1).max(2); }
        }
        let v = derive(&sel, new_inst, first, heads, table, &[], &[]);
        self.publish(v);
        Some(())
    }
    fn set_readable(&mut self, slot: usize, r: bool) { if let Some(s) = self.ring[slot].as_mut() { s.readable = r; } }
    fn set_recs(&mut self, inst: u32, txg: u64, r: bool) {
        for x in &mut self.recs { if x.inst == inst && x.txg == txg { x.readable = r; } }
    }

    fn apply(&mut self, cfg: &Config, op: Op) -> Option<Step> {
        let cur = self.cur.clone();
        let txg = cur.txg + 1;
        let mut step = Step::default();
        match op {
            Op::Create(t) => {
                let h = cur.head(t)?;
                let n = h.wm;
                let mut heads = cur.heads.clone();
                let hh = heads.iter_mut().find(|x| x.tree == t).expect("在");
                hh.objs.retain(|&(m, _)| m != n);
                hh.objs.push((n, txg));
                hh.objs.sort_unstable();
                hh.wm = n + 1;
                let o = (t, n, txg);
                let v = derive(&cur, cur.inst, txg, heads, cur.table.clone(), &[o], &[]);
                self.written.push((o, cur.inst));
                self.publish(v);
                step.created.push(o);
            }
            Op::Delete(t) => {
                let h = cur.head(t)?;
                let victim = h.objs.iter().copied().filter(|&(n, _)| n >= 2).max()?;
                let mut heads = cur.heads.clone();
                heads.iter_mut().find(|x| x.tree == t).expect("在").objs.retain(|&p| p != victim);
                let v = derive(&cur, cur.inst, txg, heads, cur.table.clone(), &[], &[]);
                self.publish(v);
            }
            Op::Clone => {
                if cur.heads.len() != 1 { return None; }
                let origin = cur.heads[0].clone();
                let mut heads = cur.heads.clone();
                heads.push(Head { tree: 2, wm: origin.wm, objs: origin.objs.clone() });
                let inherited: Vec<Obj> = origin.objs.iter().map(|&(n, b)| (2, n, b)).collect();
                let mut v = derive(&cur, cur.inst, txg, heads, cur.table.clone(), &[], &inherited);
                // C3rL 按树分的那一份：克隆把 origin 的整份发号记录抄给新树（发过又删了的也抄，代价按条数记）。
                let copied: Vec<Obj> = cur.hist.log.iter().filter(|o| o.0 == 1).map(|&(_, n, b)| (2, n, b)).collect();
                step.cloned_log_entries = u64::try_from(copied.len()).expect("小");
                for o in copied { ins(&mut v.hist.log, o); }
                for &o in &inherited { self.written.push((o, cur.inst)); }
                self.publish(v);
            }
            Op::Rollback(k) | Op::RollbackCurHidden(k) => {
                let hide_cur = matches!(op, Op::RollbackCurHidden(_));
                let cur_slot = self.slot_of(&cur)?;
                if hide_cur { self.set_readable(cur_slot, false); }
                let trees = cur.trees();
                let mut cands: Vec<Version> = self
                    .readable()
                    .into_iter()
                    .filter(|v| !(v.txg == cur.txg && v.inst == cur.inst))
                    .filter(|v| !abandoned_by(&cur.table, v.inst, v.txg))
                    .filter(|v| v.trees() == trees)
                    .cloned()
                    .collect();
                cands.sort_by_key(|v| std::cmp::Reverse((v.txg, v.inst)));
                let Some(target) = cands.get(usize::from(k) - 1).cloned() else {
                    if hide_cur { self.set_readable(cur_slot, true); }
                    return None;
                };
                let ring: Vec<Version> = self.readable().into_iter().cloned().collect();
                if hide_cur { self.set_readable(cur_slot, true); }
                let mut heads = Vec::new();
                let mut h_cur = Vec::new();
                let mut h_new = Vec::new();
                for t in &trees {
                    let all = ring.iter().filter(|v| v.head(*t).is_some()).map(|v| v.wm(*t)).max();
                    let valid = ring.iter().filter(|v| v.head(*t).is_some() && !abandoned_by(&cur.table, v.inst, v.txg)).map(|v| v.wm(*t)).max();
                    let c = cur.wm(*t);
                    if all.unwrap_or(0) > c && valid.unwrap_or(0) <= c { step.jump_from_abandoned_only = true; }
                    if valid.unwrap_or(0) > c { step.jump_from_valid = true; }
                    let set = if cfg.rule == WmRule::Opt1 { all } else { valid };
                    let w = match cfg.arm {
                        Arm::Product | Arm::V3 => c.max(set.unwrap_or(0)),
                        Arm::V1 => target.wm(*t),
                        Arm::VR => set.unwrap_or_else(|| target.wm(*t)),
                    };
                    heads.push(Head { tree: *t, wm: w, objs: target.head(*t).expect("头集合相同").objs.clone() });
                    h_cur.push((*t, c));
                    h_new.push((*t, w));
                }
                let count = trees.iter().map(|t| (*t, u64::try_from(cur.hist.log.iter().filter(|o| o.0 == *t).count()).expect("小"))).collect();
                let count_nocopy = trees.iter().map(|t| (*t, u64::try_from(cur.hist.log_nocopy.iter().filter(|o| o.0 == *t).count()).expect("小"))).collect();
                let mut pool: Vec<(u64, u64)> = cur.hist.log.iter().filter(|o| o.0 == 1).map(|&(_, n, b)| (n, b)).collect();
                pool.extend(cur.hist.log_nocopy.iter().filter(|o| o.0 != 1).map(|&(_, n, b)| (n, b)));
                pool.sort_unstable();
                pool.dedup();
                let mut base = cur.clone();
                base.hist.lin.push(Lin { t_rb: txg, t_old: target.txg, h_cur, h_new, count, count_nocopy, count_pool: u64::try_from(pool.len()).expect("小") });
                let extra: Vec<Obj> = target.heads.iter().flat_map(|h| h.objs.iter().map(move |&(n, b)| (h.tree, n, b))).collect();
                let v = derive(&base, cur.inst, txg, heads, cur.table.clone(), &[], &extra);
                self.publish(v);
                step.rollback = true;
            }
            Op::Empty => {
                let v = derive(&cur, cur.inst, txg, cur.heads.clone(), cur.table.clone(), &[], &[]);
                self.publish(v);
            }
            Op::Mount => { self.recover(cfg)?; }
            Op::CrashHide | Op::CrashHideRR | Op::CrashHide2 => {
                let newest = self.newest()?.clone();
                if newest.txg != cur.txg || newest.inst != cur.inst { return None; }
                let mut hidden = vec![self.slot_of(&cur)?];
                if op == Op::CrashHide2 {
                    let older = self.readable().into_iter().filter(|v| !(v.txg == cur.txg && v.inst == cur.inst)).last()?.clone();
                    // 第二条藏的根要是这条时间线上 cur 的上一版（同实例、txg 连着）。
                    if older.inst != cur.inst || older.txg + 1 != cur.txg { return None; }
                    hidden.push(self.slot_of(&older)?);
                }
                for &s in &hidden { self.set_readable(s, false); }
                if op == Op::CrashHideRR { self.set_recs(cur.inst, cur.txg, false); }
                if self.newest().is_none() {
                    for &s in &hidden { self.set_readable(s, true); }
                    self.set_recs(cur.inst, cur.txg, true);
                    return None;
                }
                self.recover(cfg)?;
                for &s in &hidden { self.set_readable(s, true); }
                self.set_recs(cur.inst, cur.txg, true);
            }
            Op::CrashMidCreate => {
                let n = cur.wm(1);
                let o = (1, n, txg);
                self.recs.push(Rec { inst: cur.inst, txg, readable: true });
                self.written.push((o, cur.inst));
                self.recover(cfg)?;
            }
        }
        Some(step)
    }
}

const FORMS: [&str; 19] = [
    "I-9.29", "I-9.6", "C2a-create", "C2a-skip", "C2a-src", "C2aL", "C2aL-pool", "C3rL", "C3rL-1log", "C3rL-nocopy",
    "C4g", "C4g-pool", "C2i", "C2i-pool", "C2ig", "C2ig-pool", "I-3.12", "X-txg", "C2aL+C3rL",
];
const NF: usize = FORMS.len();

fn two_births(objs: &[Obj]) -> bool { objs.windows(2).any(|w| w[0].0 == w[1].0 && w[0].1 == w[1].1 && w[0].2 != w[1].2) }

/// I-9.30 草案 ①②③（第一轮正推），按树判；`skip_cover` = 「跳升之后不判 ③」那种容许读法。
fn i930(rows: &[(u32, u64, u64)], v: &Version, skip_cover: bool) -> bool {
    v.heads.iter().any(|h| {
        let r: Vec<(u64, u64)> = rows.iter().filter(|x| x.0 == h.tree).map(|x| (x.1, x.2)).collect();
        let floor = r.iter().any(|&(_, hh)| hh > h.wm);
        let cut = h.objs.iter().any(|&(n, b)| r.iter().any(|&(g, hh)| hh > n && b > g));
        let cover = !skip_cover && h.wm > 2 && !r.iter().any(|&(g, hh)| hh >= h.wm && g <= v.txg);
        floor || cut || cover
    })
}

fn max_h(rows: &[(u32, u64, u64)], tree: Option<u32>, upto: u64) -> u64 {
    rows.iter().filter(|x| tree.is_none_or(|t| x.0 == t) && x.1 <= upto).map(|x| x.2).max().unwrap_or(0)
}

fn verdicts(state: &State, v: &Version) -> [bool; NF] {
    let readable = state.readable();
    let cands: Vec<&Version> = readable.iter().copied().filter(|o| !abandoned_by(&v.table, o.inst, o.txg)).collect();
    let mut live: Vec<Obj> = Vec::new();
    for o in &cands { for h in &o.heads { for &(n, b) in &h.objs { ins(&mut live, (h.tree, n, b)); } } }
    let i929 = two_births(&live);
    let i96 = live.iter().any(|&(t, n, _)| v.head(t).is_some_and(|h| n >= h.wm));
    let hs = &v.hist;
    let c2a_create = i930(&hs.rows_create, v, false);
    let jumped = hs.lin.iter().any(|l| l.h_new.iter().zip(&l.h_cur).any(|(a, b)| a.1 > b.1));
    let c2a_skip = i930(&hs.rows_create, v, jumped);
    // 「容许跳升只来自被抛弃的根」照字面核来源：跳升那一格只在环里还读得出一条按最新根的表被抛弃、该树水位 ≥ H_new 的根时才当一行。
    let mut rows_src = hs.rows_create.clone();
    for l in &hs.lin {
        for (&(t, hn), &(_, hc)) in l.h_new.iter().zip(&l.h_cur) {
            if hn > hc && readable.iter().any(|r| abandoned_by(&v.table, r.inst, r.txg) && r.head(t).is_some_and(|h| h.wm >= hn)) {
                rows_src.push((t, l.t_rb, hn));
            }
        }
    }
    let c2a_src = i930(&rows_src, v, false);
    let pair_eq = hs.lin.iter().any(|l| l.h_cur.iter().any(|&(t, hc)| hc == 0 || max_h(&hs.rows_all, Some(t), l.t_rb - 1) != hc));
    let c2al = i930(&hs.rows_all, v, false) || pair_eq;
    let pair_pool = hs.lin.iter().any(|l| {
        let hc = l.h_cur.iter().map(|x| x.1).max().unwrap_or(0);
        hc == 0 || max_h(&hs.rows_all, None, l.t_rb - 1) != hc
    });
    let c2al_pool = i930(&hs.rows_all, v, false) || pair_pool;
    let c3r_tree = |log: &[Obj], counts: &dyn Fn(&Lin) -> Vec<(u32, u64)>| -> bool {
        two_births(log)
            || log.iter().any(|&(t, n, _)| v.head(t).is_some_and(|h| n >= h.wm))
            || v.heads.iter().any(|h| h.objs.iter().any(|&(n, b)| log.binary_search(&(h.tree, n, b)).is_err()))
            || hs.lin.iter().any(|l| counts(l).iter().any(|&(t, c)| {
                c == 0 || u64::try_from(log.iter().filter(|o| o.0 == t && o.2 < l.t_rb).count()).expect("小") != c
            }))
    };
    let c3rl = c3r_tree(&hs.log, &|l: &Lin| l.count.clone());
    let c3rl_nocopy = c3r_tree(&hs.log_nocopy, &|l: &Lin| l.count_nocopy.clone());
    let mut pooled: Vec<(u64, u64)> = hs.log.iter().filter(|o| o.0 == 1).map(|&(_, n, b)| (n, b)).collect();
    pooled.extend(hs.log_nocopy.iter().filter(|o| o.0 != 1).map(|&(_, n, b)| (n, b)));
    pooled.sort_unstable();
    pooled.dedup();
    let wm_max = v.heads.iter().map(|h| h.wm).max().unwrap_or(0);
    let c3rl_1log = pooled.windows(2).any(|w| w[0].0 == w[1].0 && w[0].1 != w[1].1)
        || pooled.iter().any(|&(n, _)| n >= wm_max)
        || v.heads.iter().any(|h| h.objs.iter().any(|p| pooled.binary_search(p).is_err()))
        || hs.lin.iter().any(|l| l.count_pool == 0 || u64::try_from(pooled.iter().filter(|p| p.1 < l.t_rb).count()).expect("小") != l.count_pool);
    let c4g = hs.lin.iter().any(|l| l.h_cur.iter().any(|&(t, w)| {
        v.head(t).is_some_and(|h| h.wm < w || h.objs.iter().any(|&(n, b)| n < w && b > l.t_rb))
    }));
    let c4g_pool = hs.lin.iter().any(|l| {
        let w = l.h_cur.iter().map(|x| x.1).max().unwrap_or(0);
        wm_max < w || v.heads.iter().any(|h| h.objs.iter().any(|&(n, b)| n < w && b > l.t_rb))
    });
    let c2i = hs.peak.iter().any(|&(_, t, p, g)| v.head(t).is_some_and(|h| p > h.wm || h.objs.iter().any(|&(n, b)| n < p && b > g)));
    let c2i_pool = hs.peak_pool.iter().any(|&(_, p, g)| p > wm_max || v.heads.iter().any(|h| h.objs.iter().any(|&(n, b)| n < p && b > g)));
    let i312 = readable.iter().any(|r| r.table.iter().any(|&(_, t)| t >= r.txg));
    let first_j = readable.iter().filter(|r| r.inst == v.inst).map(|r| r.txg).min().unwrap_or(v.txg);
    let x_txg = readable.iter().any(|r| r.inst != v.inst && abandoned_by(&v.table, r.inst, r.txg) && r.txg >= first_j)
        || state.recs.iter().any(|r| r.readable && r.inst != v.inst && abandoned_by(&v.table, r.inst, r.txg) && r.txg >= first_j);
    [
        i929, i96, c2a_create, c2a_skip, c2a_src, c2al, c2al_pool, c3rl, c3rl_1log, c3rl_nocopy, c4g, c4g_pool, c2i, c2i_pool,
        c2i || c4g, c2i_pool || c4g_pool, i312, x_txg, c2al || c3rl,
    ]
}

const KINDS: [&str; 9] = [
    "false_positive", "event_missed", "residual_missed", "hole_event_missed", "hole_residual_missed",
    "false_positive_after_jump", "event_missed_after_jump", "red_on_identity_collision", "event_missed_without_jump",
];

#[derive(Default, Clone)]
struct Tally {
    images: u64,
    legal: u64,
    events: u64,
    residuals: u64,
    hole_events: u64,
    hole_residuals: u64,
    rollbacks: u64,
    jumps_abandoned_only: u64,
    jumps_valid: u64,
    collisions: u64,
    collisions_visible: u64,
    newest_not_cur: u64,
    cloned_log_entries: u64,
    clones: u64,
    events_after_jump: u64,
    counts: Vec<[u64; KINDS.len()]>,
    examples: Vec<Vec<Option<Vec<u8>>>>,
    coll_example: Option<Vec<u8>>,
}

#[derive(Default, Clone, Copy)]
struct Path { jumped: bool }

fn better(seq: &[u8], kept: &Option<Vec<u8>>) -> bool { kept.as_ref().is_none_or(|k| (seq.len(), seq) < (k.len(), k.as_slice())) }

impl Tally {
    fn new() -> Tally { Tally { counts: vec![[0; KINDS.len()]; NF], examples: vec![vec![None; KINDS.len()]; NF], ..Tally::default() } }
    fn hit(&mut self, f: usize, k: usize, seq: &[u8]) {
        self.counts[f][k] += 1;
        if better(seq, &self.examples[f][k]) { self.examples[f][k] = Some(seq.to_vec()); }
    }
    fn judge(&mut self, state: &State, step: &Step, path: Path, seq: &[u8]) {
        self.images += 1;
        if step.rollback { self.rollbacks += 1; }
        if step.jump_from_abandoned_only { self.jumps_abandoned_only += 1; }
        if step.jump_from_valid { self.jumps_valid += 1; }
        if step.cloned_log_entries > 0 { self.clones += 1; self.cloned_log_entries += step.cloned_log_entries; }
        let Some(v) = state.newest() else { return };
        let is_cur = v.txg == state.cur.txg && v.inst == state.cur.inst;
        if !is_cur { self.newest_not_cur += 1; }
        let red = verdicts(state, v);
        let reissue = two_births(&v.issued);
        let regressed = v.issued.iter().any(|&(t, n, _)| v.head(t).is_some_and(|h| n >= h.wm));
        let legal = !reissue && !regressed;
        if legal { self.legal += 1; }
        let event = is_cur && step.created.iter().any(|&(t, n, b)| v.issued.iter().any(|&(t2, n2, b2)| t2 == t && n2 == n && b2 != b));
        let residual = !event && v.heads.iter().any(|h| h.objs.iter().any(|&(n, b)| v.issued.iter().any(|&(t2, n2, b2)| t2 == h.tree && n2 == n && b2 < b)));
        let hole = !red[0] && !red[1];
        if event { self.events += 1; if hole { self.hole_events += 1; } if path.jumped { self.events_after_jump += 1; } }
        if residual { self.residuals += 1; if hole { self.hole_residuals += 1; } }
        // 身份相撞：这条时间线上的 (树, 号, 出生代) 与另一个实例写过的逐字相同（不是同一条时间线上的重发）。
        let collided: Vec<Obj> = v
            .issued
            .iter()
            .copied()
            .filter(|o| {
                let mut insts: Vec<u32> = state.written.iter().filter(|(w, _)| w == o).map(|(_, i)| *i).collect();
                insts.sort_unstable();
                insts.dedup();
                insts.len() >= 2
            })
            .collect();
        let collision = !collided.is_empty();
        if collision {
            self.collisions += 1;
            let visible = collided.iter().any(|&o| {
                state.readable().iter().any(|r| abandoned_by(&v.table, r.inst, r.txg) && r.heads.iter().any(|h| h.tree == o.0 && h.objs.contains(&(o.1, o.2))))
                    || state.recs.iter().any(|r| r.readable && abandoned_by(&v.table, r.inst, r.txg) && r.txg == o.2)
            });
            if visible { self.collisions_visible += 1; }
            if better(seq, &self.coll_example) { self.coll_example = Some(seq.to_vec()); }
        }
        for f in 0..NF {
            if legal && red[f] { self.hit(f, 0, seq); if path.jumped { self.hit(f, 5, seq); } }
            if event && !red[f] {
                self.hit(f, 1, seq);
                if hole { self.hit(f, 3, seq); }
                if path.jumped { self.hit(f, 6, seq); } else { self.hit(f, 8, seq); }
            }
            if residual && !red[f] { self.hit(f, 2, seq); if hole { self.hit(f, 4, seq); } }
            if collision && red[f] { self.hit(f, 7, seq); }
        }
    }
}

fn explore(cfg: &Config, state: &State, path: Path, seq: &mut Vec<u8>, t: &mut Tally, leaves: &mut u64) {
    if seq.len() == cfg.depth { *leaves += 1; return; }
    for (i, op) in cfg.ops.iter().enumerate() {
        let mut next = state.clone();
        let Some(step) = next.apply(cfg, *op) else { continue };
        let np = Path { jumped: path.jumped || step.jump_from_abandoned_only };
        seq.push(u8::try_from(i).expect("小"));
        t.judge(&next, &step, np, seq);
        explore(cfg, &next, np, seq, t, leaves);
        seq.pop();
    }
}

fn render(state: &State) -> String {
    let v = state.newest().expect("有根");
    let heads: Vec<String> = v.heads.iter().map(|h| {
        let o: Vec<String> = h.objs.iter().map(|(n, b)| format!("{n}@{b}")).collect();
        format!("t{}:wm{}:[{}]", h.tree, h.wm, o.join(","))
    }).collect();
    let rows: Vec<String> = v.hist.rows_all.iter().map(|(t, g, h)| format!("t{t}:{g}:{h}")).collect();
    let lin: Vec<String> = v.hist.lin.iter().map(|l| format!("{}<-{} Hcur{:?} Hnew{:?}", l.t_rb, l.t_old, l.h_cur, l.h_new)).collect();
    let ring: Vec<String> = state.ring.iter().flatten().map(|s| {
        let w: Vec<String> = s.v.heads.iter().map(|h| format!("{}", h.wm)).collect();
        format!("{}:{}:wm{}", s.v.inst, s.v.txg, w.join("/"))
    }).collect();
    let table: Vec<String> = v.table.iter().map(|(i, t)| format!("{i}:{t}")).collect();
    let recs: Vec<String> = state.recs.iter().map(|r| format!("{}:{}", r.inst, r.txg)).collect();
    format!(
        "newest={}:{} cur={}:{} heads=[{}] table=[{}] rows_all=[{}] lin=[{}] ring=[{}] records=[{}] issued={:?}",
        v.inst, v.txg, state.cur.inst, state.cur.txg, heads.join(" "), table.join(","), rows.join(","), lin.join(";"), ring.join(","), recs.join(","), v.issued
    )
}

fn trace(cfg: &Config, seq: &[u8]) -> Vec<String> {
    let mut s = State::initial(cfg.ring);
    let mut out = vec![format!("step=start {}", render(&s))];
    for (i, idx) in seq.iter().enumerate() {
        let op = cfg.ops[usize::from(*idx)];
        let step = s.apply(cfg, op).expect("复放");
        let red = verdicts(&s, s.newest().expect("有根"));
        let names: Vec<&str> = FORMS.iter().zip(red).filter(|(_, r)| *r).map(|(n, _)| *n).collect();
        out.push(format!("step={i} op={} jump_abandoned={} {} red=[{}]", op.code(), u8::from(step.jump_from_abandoned_only), render(&s), names.join(",")));
    }
    out
}

fn label(c: &Config) -> String {
    format!("group={} rule={:?} arm={:?} first_txg={:?} ring={} depth={}", c.group, c.rule, c.arm, c.ft, c.ring, c.depth)
}

fn configs() -> Vec<Config> {
    let base = Config { rule: WmRule::Opt1, arm: Arm::Product, ft: FirstTxg::Product, ring: 4, depth: 8, ops: &K2_OPS, group: "k2" };
    let mut out = Vec::new();
    for rule in [WmRule::Opt1, WmRule::Opt2] {
        for arm in [Arm::Product, Arm::V1, Arm::VR, Arm::V3] { out.push(Config { rule, arm, ..base }); }
    }
    for rule in [WmRule::Opt1, WmRule::Opt2] {
        for arm in [Arm::Product, Arm::V1] { out.push(Config { rule, arm, ring: 6, depth: 7, group: "k2ring6", ..base }); }
    }
    for (rule, arm) in [(WmRule::Opt1, Arm::Product), (WmRule::Opt1, Arm::V1), (WmRule::Opt2, Arm::Product)] {
        out.push(Config { rule, arm, ops: &CLONE_OPS, group: "clone", ..base });
    }
    for ft in [FirstTxg::Product, FirstTxg::RingRootsOnly, FirstTxg::SelectedPlusOne] {
        out.push(Config { ft, ops: &FT_OPS, group: "ft", ..base });
    }
    out
}

fn run(cfg: &Config) {
    let start = std::time::Instant::now();
    let mut t = Tally::new();
    let mut leaves = 0;
    explore(cfg, &State::initial(cfg.ring), Path::default(), &mut Vec::new(), &mut t, &mut leaves);
    let l = label(cfg);
    println!(
        "W3R3S2MODEL {l} full_length_histories={leaves} images={} legal_images={} events={} residual_images={} hole_events={} hole_residual_images={} rollbacks={} jumps_from_abandoned_roots_only={} jumps_from_a_root_not_abandoned={} events_after_a_jump={} identity_collision_images={} identity_collision_images_other_copy_visible={} newest_root_not_cur_images={} clones={} cloned_log_entries={} elapsed_s={:.1}",
        t.images, t.legal, t.events, t.residuals, t.hole_events, t.hole_residuals, t.rollbacks, t.jumps_abandoned_only, t.jumps_valid,
        t.events_after_jump, t.collisions, t.collisions_visible, t.newest_not_cur, t.clones, t.cloned_log_entries, start.elapsed().as_secs_f64()
    );
    for (f, name) in FORMS.iter().enumerate() {
        let c: Vec<String> = KINDS.iter().enumerate().map(|(k, kn)| format!("{kn}={}", t.counts[f][k])).collect();
        println!("W3R3S2COUNT {l} form={name} {}", c.join(" "));
    }
    let show = |seq: &Vec<u8>| seq.iter().map(|i| cfg.ops[usize::from(*i)].code()).collect::<Vec<_>>().join(",");
    for (f, name) in FORMS.iter().enumerate() {
        for (k, kn) in KINDS.iter().enumerate() {
            if let Some(seq) = &t.examples[f][k] { println!("W3R3S2EXAMPLE {l} form={name} kind={kn} ops=[{}]", show(seq)); }
        }
    }
    if let Some(seq) = &t.coll_example {
        println!("W3R3S2EXAMPLE {l} form=- kind=identity_collision ops=[{}]", show(seq));
        for line in trace(cfg, seq) { println!("W3R3S2TRACE {l} kind=identity_collision {line}"); }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let all = configs();
    match args.first().map(String::as_str) {
        Some("list") => { for (i, c) in all.iter().enumerate() { println!("{i} {}", label(c)); } }
        Some("run") => {
            let i: usize = args.get(1).expect("序号").parse().expect("序号");
            run(&all[i]);
        }
        Some("trace") => {
            // trace <序号> <op,op,...>（op 用 code 写法）
            let cfg = all[args.get(1).expect("序号").parse::<usize>().expect("序号")];
            let names: Vec<String> = cfg.ops.iter().map(|o| o.code()).collect();
            let seq: Vec<u8> = args.get(2).expect("步").split(',').map(|x| u8::try_from(names.iter().position(|n| n == x).expect("认得的步")).expect("小")).collect();
            for line in trace(&cfg, &seq) { println!("W3R3S2TRACE {} {line}", label(&cfg)); }
        }
        _ => { eprintln!("用法：list | run <序号> | trace <序号> <步,步,…>"); std::process::exit(2); }
    }
    let _ = BTreeMap::<u8, u8>::new();
}
