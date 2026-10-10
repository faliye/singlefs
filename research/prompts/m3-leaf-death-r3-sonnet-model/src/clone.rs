//! 克隆头共享叶的有界穷举：origin（头 0）建一棵叶容量 2 的树，在 fork 点之后 origin（头 0）与克隆头（头 1）各自建 / unlink，
//! 长度 ≤ MAX_LEN 的全部序列各跑一遍，每一步之后对两个头各重建一遍视图、与各自的真值比。
//! 规则照 kb：克隆头首次 COW 一片共享叶 = 新身份（本头树, 原容器号, 本次发布代号）并给旧身份写退役记录（D8 已定项 6，C113 定案 P6）；
//! 每个头自己一份退役记录、墓碑记录（头 ID = 本头 inode 树）；读克隆头时只收 origin 在 fork 之前写的版本、origin 的退役 / 墓碑记录只认死亡 txg ≤ fork 的。
//! 消融：GlobalRetire（退役记录不分头，全池一份）、GlobalTomb（墓碑不分头）、NoForkBound（读克隆头时不按 fork 截 origin 的版本）。
use std::collections::BTreeMap;

const CAP: usize = 2;
const MAX_LEN: usize = 6;

type Id = (u8, u64, u64); // (出生树, 容器号, 容器出生代)

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CloneCand { Today, Ret, Persist, Tsz }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Ablation { None, GlobalRetire, GlobalTomb, NoForkBound }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct R { ino: u64, obirth: u64, nlink: u32 }

#[derive(Clone, Debug)]
struct Ver { id: Id, vbirth: u64, recs: Vec<R> }

#[derive(Clone, Debug)]
struct Leaf { id: Id, recs: Vec<R>, dirty: bool }

#[derive(Clone, Debug, Default)]
struct Head { leaves: Vec<Leaf>, rets: Vec<(Id, u64)>, tombs: Vec<(u64, u64, u64)>, queue: Option<u64>, watermark: u64 }

#[derive(Clone, Debug)]
struct W {
    cand: CloneCand,
    heads: [Head; 2],
    units: Vec<Ver>,
    txg: u64,
    fork: u64,
    writes: BTreeMap<&'static str, u64>,
}

#[derive(Clone, Copy, Debug)]
enum Op { Create(usize), UnlinkFirst(usize), UnlinkLast(usize) }

impl W {
    fn new(cand: CloneCand) -> Self {
        // origin 先建 5 个 inode：叶 [1,2] [3,4] [5]（容量 2），fork 在这之后。
        let mut w = W { cand, heads: [Head::default(), Head::default()], units: Vec::new(), txg: 0, fork: 0, writes: BTreeMap::new() };
        w.heads[0].watermark = 1;
        for _ in 0..5 { w.create(0); }
        w.fork = w.txg;
        w.heads[1] = w.heads[0].clone();
        w.writes.clear();
        w
    }

    fn publish(&mut self, tag: &'static str) {
        self.txg += 1;
        let txg = self.txg;
        for h in 0..2 {
            for l in &mut self.heads[h].leaves {
                if l.dirty {
                    l.dirty = false;
                    self.units.push(Ver { id: l.id, vbirth: txg, recs: l.recs.clone() });
                    *self.writes.entry(tag).or_default() += 1;
                }
            }
        }
    }

    /// 头 h 要改第 idx 片叶：共享叶（出生树不是本头）首次 COW 重生并给旧身份写退役记录。
    fn cow(&mut self, h: usize, idx: usize) {
        let me = h as u8;
        let leaf = &mut self.heads[h].leaves[idx];
        if leaf.id.0 != me {
            let old = leaf.id;
            leaf.id = (me, old.1, self.txg + 1);
            self.heads[h].rets.push((old, self.txg + 1));
            *self.writes.entry("p6_retire").or_default() += 1;
        }
        self.heads[h].leaves[idx].dirty = true;
    }

    fn create(&mut self, h: usize) {
        let txg = self.txg + 1;
        let ino = self.heads[h].watermark;
        self.heads[h].watermark += 1;
        let rec = R { ino, obirth: txg, nlink: 1 };
        let n = self.heads[h].leaves.len();
        if n > 0 && self.heads[h].leaves[n - 1].recs.len() < CAP {
            self.cow(h, n - 1);
            self.heads[h].leaves[n - 1].recs.push(rec);
        } else {
            self.heads[h].leaves.push(Leaf { id: (h as u8, ino, txg), recs: vec![rec], dirty: true });
        }
        self.publish("create");
    }

    fn unlink(&mut self, h: usize, ino: u64) {
        let txg = self.txg + 1;
        let (li, ri) = {
            let hd = &self.heads[h];
            let li = hd.leaves.iter().position(|l| l.recs.iter().any(|r| r.ino == ino)).unwrap();
            (li, hd.leaves[li].recs.iter().position(|r| r.ino == ino).unwrap())
        };
        let obirth = self.heads[h].leaves[li].recs[ri].obirth;
        // 意图：nlink 写成 0，墓碑（今天就写，所有候选一样）
        self.cow(h, li);
        self.heads[h].leaves[li].recs[ri].nlink = 0;
        self.heads[h].queue = Some(ino);
        self.heads[h].tombs.push((ino, obirth, txg));
        self.publish("intent");
        // 最后一批：删记录，再看这一片
        self.cow(h, li);
        self.heads[h].leaves[li].recs.remove(ri);
        self.heads[h].queue = None;
        let txg2 = self.txg + 1;
        let cand = self.cand;
        if self.heads[h].leaves[li].recs.is_empty() {
            let gone = self.heads[h].leaves[li].id;
            match cand {
                CloneCand::Persist => { /* 零条目叶一直留在叶序里：本头的身份写一版零条目 */ }
                CloneCand::Ret => { self.heads[h].leaves.remove(li); self.heads[h].rets.push((gone, txg2)); *self.writes.entry("ret_record").or_default() += 1; }
                _ => { self.heads[h].leaves.remove(li); }
            }
        } else if li + 1 < self.heads[h].leaves.len() && self.heads[h].leaves[li].recs.len() + self.heads[h].leaves[li + 1].recs.len() <= CAP
            && !self.heads[h].leaves[li + 1].recs.is_empty() {
            // 左吸收右（M2）。右片是共享叶也一样要从本头的视图里消失：照 P6 / 合并退役，写退役记录（Today 与 Tsz 只在这一处也写，因为 kb 已定「合并时右半退役」）。
            let right = self.heads[h].leaves[li + 1].clone();
            self.heads[h].leaves[li].recs.extend(right.recs.clone());
            match cand {
                CloneCand::Persist => {
                    // 右片留零条目：共享叶不能原地写（会改 origin 的身份），要重生成本头的零条目叶并给旧身份写退役记录
                    self.cow(h, li + 1);
                    self.heads[h].leaves[li + 1].recs.clear();
                }
                _ => {
                    self.heads[h].leaves.remove(li + 1);
                    self.heads[h].rets.push((right.id, txg2));
                    *self.writes.entry("merge_retire").or_default() += 1;
                }
            }
        }
        self.publish("last");
    }

    fn live(&self, h: usize) -> Vec<u64> {
        self.heads[h].leaves.iter().flat_map(|l| l.recs.iter()).filter(|r| r.nlink > 0).map(|r| r.ino).collect()
    }

    fn apply(&mut self, op: Op) -> bool {
        match op {
            Op::Create(h) => { self.create(h); true }
            Op::UnlinkFirst(h) => { let l = self.live(h); if let Some(&x) = l.first() { self.unlink(h, x); true } else { false } }
            Op::UnlinkLast(h) => { let l = self.live(h); if let Some(&x) = l.last() { self.unlink(h, x); true } else { false } }
        }
    }

    /// 头 h 的视图 vs 头 h 的真值：返回 (复活, 误判死亡, 窗内误判死亡)。
    fn judge(&self, h: usize, ab: Ablation) -> (u64, u64, bool) {
        let me = h as u8;
        let fork_bound = |v: &Ver| -> bool {
            if v.id.0 == me { return true; }
            // 别的头出生的身份：克隆头（me = 1）只收 fork 之前写的版本；origin 读自己时不收克隆头出生的身份
            if me == 1 && v.id.0 == 0 { return ab == Ablation::NoForkBound || v.vbirth <= self.fork; }
            false
        };
        let mut newest: BTreeMap<Id, &Ver> = BTreeMap::new();
        for v in &self.units {
            if !fork_bound(v) { continue; }
            let e = newest.entry(v.id).or_insert(v);
            if v.vbirth > e.vbirth { *e = v; }
        }
        // 退役集合：本头自己的；克隆头再加 origin 的、死亡 txg ≤ fork 的。GlobalRetire：全池。
        let mut retired: Vec<Id> = self.heads[h].rets.iter().map(|(i, _)| *i).collect();
        if ab == Ablation::GlobalRetire { retired = self.heads.iter().flat_map(|x| x.rets.iter().map(|(i, _)| *i)).collect(); }
        else if me == 1 { retired.extend(self.heads[0].rets.iter().filter(|(_, d)| *d <= self.fork).map(|(i, _)| *i)); }
        let mut tombed: Vec<(u64, u64)> = Vec::new();
        if self.cand == CloneCand::Tsz {
            let own = self.heads[h].tombs.iter().map(|(a, b, _)| (*a, *b));
            tombed.extend(own);
            if ab == Ablation::GlobalTomb { tombed = self.heads.iter().flat_map(|x| x.tombs.iter().map(|(a, b, _)| (*a, *b))).collect(); }
            else if me == 1 { tombed.extend(self.heads[0].tombs.iter().filter(|(_, _, d)| *d <= self.fork).map(|(a, b, _)| (*a, *b))); }
        }
        let mut inode: BTreeMap<u64, (u64, R)> = BTreeMap::new();
        for (id, v) in &newest {
            if retired.contains(id) { continue; }
            for r in &v.recs {
                if tombed.contains(&(r.ino, r.obirth)) { continue; }
                let e = inode.entry(r.ino).or_insert((v.vbirth, *r));
                if v.vbirth > e.0 { *e = (v.vbirth, *r); }
            }
        }
        let truth: BTreeMap<u64, R> = self.heads[h].leaves.iter().flat_map(|l| l.recs.iter()).map(|r| (r.ino, *r)).collect();
        let r = inode.iter().filter(|(k, (_, rec))| truth.get(k) != Some(rec)).count() as u64;
        let m = truth.keys().filter(|k| !inode.contains_key(k)).count() as u64;
        (r, m, self.heads[h].queue.is_some())
    }
}

#[derive(Default, Debug)]
struct Tally { sequences: u64, scans: u64, r_scans: [u64; 2], m_scans: [u64; 2], m_in_window: [u64; 2], sequences_with_r: u64, sequences_with_m_outside_window: u64, first_bad: Option<String>,
    writes: BTreeMap<&'static str, u64> }

fn search(cand: CloneCand, ab: Ablation) -> Tally {
    let ops: Vec<Op> = (0..2).flat_map(|h| [Op::Create(h), Op::UnlinkFirst(h), Op::UnlinkLast(h)]).collect();
    let mut t = Tally::default();
    // 深度优先：每个前缀走一遍，序列 = 到当前深度的前缀（长度 1..=MAX_LEN 的全部序列各算一次）
    fn go(w: &W, depth: usize, ops: &[Op], ab: Ablation, path: &mut Vec<Op>, t: &mut Tally) {
        if depth == MAX_LEN { return; }
        for &op in ops {
            let mut n = w.clone();
            if !n.apply(op) { continue; }
            path.push(op);
            t.sequences += 1;
            let mut seq_r = false; let mut seq_m_out = false;
            for h in 0..2 {
                let (r, m, win) = n.judge(h, ab);
                t.scans += 1;
                if r > 0 { t.r_scans[h] += 1; seq_r = true; }
                if m > 0 { t.m_scans[h] += 1; if win { t.m_in_window[h] += 1; } else { seq_m_out = true; } }
                if (r > 0 || (m > 0 && !win)) && t.first_bad.is_none() { t.first_bad = Some(format!("{path:?} head={h} r={r} m={m}")); }
            }
            if seq_r { t.sequences_with_r += 1; }
            if seq_m_out { t.sequences_with_m_outside_window += 1; }
            if depth + 1 == MAX_LEN { for (k, v) in &n.writes { *t.writes.entry(k).or_default() += v; } }
            go(&n, depth + 1, ops, ab, path, t);
            path.pop();
        }
    }
    let w = W::new(cand);
    go(&w, 0, &ops, ab, &mut Vec::new(), &mut t);
    t
}

pub fn run() {
    println!("R3 name=clone_config cap={CAP} max_len={MAX_LEN} initial_tree=[1,2][3,4][5] fork_after_initial=true ops=create/unlink_first/unlink_last on each of the two heads");
    for cand in [CloneCand::Today, CloneCand::Ret, CloneCand::Persist, CloneCand::Tsz] {
        for ab in [Ablation::None, Ablation::GlobalRetire, Ablation::GlobalTomb, Ablation::NoForkBound] {
            let t = search(cand, ab);
            let w = t.writes.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(",");
            println!("R3 name=clone cand={cand:?} ablation={ab:?} sequences={} scans={} r_scans_origin={} r_scans_clone={} m_scans_origin={} m_scans_clone={} m_in_window_origin={} m_in_window_clone={} sequences_with_r={} sequences_with_m_outside_window={} writes_at_full_depth=[{w}] first_bad={}",
                t.sequences, t.scans, t.r_scans[0], t.r_scans[1], t.m_scans[0], t.m_scans[1], t.m_in_window[0], t.m_in_window[1], t.sequences_with_r, t.sequences_with_m_outside_window,
                t.first_bad.clone().unwrap_or_else(|| "none".into()).replace(' ', "_"));
        }
    }
}
