//! K1「克隆头共享叶」：两个头的小模型，前缀与故障固定、用户动作逐序列全枚举。
//! - 前缀：origin O 新池建 5 个 inode（叶容量 3：[1,2,3][4,5]），再在 O 上做 0–2 步（建 / 覆盖写 / unlink 到底，逐序列全取），
//!   然后在 txg F 给 O 拍快照、建克隆头 C（C 的 inode 树共享 O 那一刻的叶；C 的墓碑树、退役树是自己的新树，类型 1 / 3 容器归写者树独占，
//!   C 读 O 的那几棵树只读快照那一刻（≤ F）的版本）；C 的 inode 号水位 = O 那一刻的水位。
//! - 分叉之后：两个头上的 0–3 步（每步：哪个头 × 建一个 / 覆盖写某号 / unlink 某号到底），逐序列全取。
//! - 盘：G-宽（旧版本一个都不盖，对复活最不利）；没有崩溃与回退（这两样第二轮与本轮主模型已扫，这里只看跨头）。
//! - 写路径（每个候选一条）：C1 退役记录（删空、被吸收、跨头首次 COW 都写，D8 已定项 6 原文）；C2lit 零条目版本、跨头首次 COW 不写任何东西；
//!   C2ret 零条目版本 + 跨头首次 COW 照 D8 写退役记录；TSZ 意图那次写整对象墓碑，叶删空 / 被吸收直接摘。
//! - 读法：每个头的视图从盘上全部单元重建。前提 A（有克隆祖先表）：C 的视图收出生树 = C 的单元 + 出生树 = O 且诞生代号 ≤ F 的单元；
//!   前提 B（没有祖先表，C 收出生树 = O 的全部单元）是共用前提坏掉的对照。C2 另有一个读法 C2num：C 的视图里同一个容器号只要有出生树 = C 的身份，
//!   出生树 = O 的那几个身份整个不收（「号接替」，没有退役记录时代替 D8 那一句的读法，被攻过零轮）。
use std::collections::{BTreeMap, HashMap, HashSet};

const CAP: usize = 3;
const O: u8 = 1;
const C: u8 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
struct Id { tree: u8, num: u64, gen: u64 }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct R { ino: u64, obirth: u64, nlink: u32, size: u64 }

#[derive(Clone, Debug)]
struct L { id: Id, recs: Vec<R> }

#[derive(Clone, Debug)]
enum P { Leaf(Id, Vec<R>), Ret(u8, Vec<(Id, u64)>), Tomb(u8, Vec<(u64, u64, u64)>) }

#[derive(Clone, Debug)]
struct U { birth: u64, p: P }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Path { C1, C2lit, C2ret, Tsz }

#[derive(Clone, Debug)]
struct Head { tree: u8, leaves: Vec<L>, wm: u64, rets: Vec<(Id, u64)>, tombs: Vec<(u64, u64, u64)>, queue: Option<u64> }

#[derive(Clone, Debug)]
struct W { path: Path, disk: Vec<U>, txg: u64, heads: Vec<Head>, fork: Option<u64> }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Op { Create(u8), Write(u8, u64), Unlink(u8, u64) }

impl W {
    fn new(path: Path) -> Self {
        let mut w = W { path, disk: Vec::new(), txg: 0, heads: vec![Head { tree: O, leaves: Vec::new(), wm: 1, rets: Vec::new(), tombs: Vec::new(), queue: None }], fork: None };
        for _ in 0..5 { w.create(0); }
        w
    }
    fn hi(&self, tree: u8) -> usize { self.heads.iter().position(|h| h.tree == tree).unwrap() }
    /// 一次发布：`dirty` 里的叶下标重写（跨头首次 COW 换身份），退役 / 墓碑容器有新记录就重写。
    fn publish(&mut self, h: usize, mut dirty: Vec<usize>, ret_new: bool, tomb_new: bool) {
        self.txg += 1;
        let t = self.txg;
        dirty.sort(); dirty.dedup();
        let tree = self.heads[h].tree;
        let mut ret_new = ret_new;
        for i in dirty {
            if i >= self.heads[h].leaves.len() { continue; }
            let old = self.heads[h].leaves[i].id;
            if old.tree != tree {
                let new = Id { tree, num: old.num, gen: t };
                self.heads[h].leaves[i].id = new;
                if matches!(self.path, Path::C1 | Path::C2ret) { self.heads[h].rets.push((old, t)); ret_new = true; }
            }
            let l = &self.heads[h].leaves[i];
            self.disk.push(U { birth: t, p: P::Leaf(l.id, l.recs.clone()) });
        }
        if ret_new { self.disk.push(U { birth: t, p: P::Ret(tree, self.heads[h].rets.clone()) }); }
        if tomb_new { self.disk.push(U { birth: t, p: P::Tomb(tree, self.heads[h].tombs.clone()) }); }
    }
    fn create(&mut self, h: usize) {
        let ino = self.heads[h].wm;
        self.heads[h].wm += 1;
        let r = R { ino, obirth: self.txg + 1, nlink: 1, size: 0 };
        let tree = self.heads[h].tree;
        let n = self.heads[h].leaves.len();
        if n > 0 && self.heads[h].leaves[n - 1].recs.len() < CAP {
            self.heads[h].leaves[n - 1].recs.push(r);
            self.publish(h, vec![n - 1], false, false);
        } else {
            self.heads[h].leaves.push(L { id: Id { tree, num: ino, gen: self.txg + 1 }, recs: vec![r] });
            self.publish(h, vec![n], false, false);
        }
    }
    fn find(&self, h: usize, ino: u64) -> Option<(usize, usize)> {
        self.heads[h].leaves.iter().enumerate().find_map(|(i, l)| l.recs.iter().position(|r| r.ino == ino).map(|k| (i, k)))
    }
    fn write(&mut self, h: usize, ino: u64) {
        let (i, k) = self.find(h, ino).unwrap();
        self.heads[h].leaves[i].recs[k].size += 1;
        self.publish(h, vec![i], false, false);
    }
    /// unlink 到底：意图那一次（nlink 0；TSZ 写墓碑），之后最后一批（删记录、删空 / 合并）。每次发布之后都判（由调用方的回调）。
    fn unlink(&mut self, h: usize, ino: u64, check: &mut dyn FnMut(&W)) {
        let (i, k) = self.find(h, ino).unwrap();
        let r = &mut self.heads[h].leaves[i].recs[k];
        r.nlink = 0;
        let ob = r.obirth;
        self.heads[h].queue = Some(ino);
        let tomb = self.path == Path::Tsz;
        if tomb { let t = self.txg + 1; self.heads[h].tombs.push((ino, ob, t)); }
        self.publish(h, vec![i], false, tomb);
        check(self);
        let (i, k) = self.find(h, ino).unwrap();
        self.heads[h].leaves[i].recs.remove(k);
        self.heads[h].queue = None;
        let stubs = matches!(self.path, Path::C2lit | Path::C2ret);
        let retires = self.path == Path::C1;
        let tree = self.heads[h].tree;
        let t = self.txg + 1;
        let leaves = &mut self.heads[h].leaves;
        let mut dirty = vec![i];
        let mut ret_new = false;
        if leaves[i].recs.is_empty() {
            if !stubs {
                let gone = leaves.remove(i);
                if retires { self.heads[h].rets.push((Self::effective(gone.id, tree, t), t)); ret_new = true; }
                dirty.clear();
            }
        } else if let Some(j) = (i + 1..leaves.len()).find(|&j| !leaves[j].recs.is_empty()).filter(|&j| leaves[i].recs.len() + leaves[j].recs.len() <= CAP) {
            let moved = std::mem::take(&mut leaves[j].recs);
            leaves[i].recs.extend(moved);
            if stubs { dirty.push(j); } else {
                let gone = leaves.remove(j);
                if retires { self.heads[h].rets.push((Self::effective(gone.id, tree, t), t)); ret_new = true; }
            }
        }
        self.publish(h, dirty, ret_new, false);
        check(self);
    }
    /// 被摘掉的叶若还是别的头的身份（没在本头 COW 过），退役记录点名的就是那个身份本身。
    fn effective(id: Id, _tree: u8, _t: u64) -> Id { id }
    fn fork(&mut self) {
        self.fork = Some(self.txg);
        let o = self.heads[0].clone();
        self.heads.push(Head { tree: C, leaves: o.leaves.clone(), wm: o.wm, rets: Vec::new(), tombs: Vec::new(), queue: None });
    }
    fn apply(&mut self, op: Op, check: &mut dyn FnMut(&W)) -> bool {
        match op {
            Op::Create(t) => { let h = self.hi(t); self.create(h); check(self); }
            Op::Write(t, x) => { let h = self.hi(t); if self.find(h, x).is_none() { return false; } self.write(h, x); check(self); }
            Op::Unlink(t, x) => { let h = self.hi(t); if self.find(h, x).is_none() { return false; } self.unlink(h, x, check); }
        }
        true
    }

    /// 一个头的视图：从盘上重建，交回 (r, m, q)。`ancestry` = 前提 A；`num_takeover` = C2num 读法。
    fn judge(&self, view: u8, ancestry: bool, num_takeover: bool) -> (u64, u64, u64) {
        let f = self.fork.unwrap_or(u64::MAX);
        let eligible = |tree: u8, birth: u64| tree == view || (view == C && tree == O && (!ancestry || birth <= f));
        let mut leaf: HashMap<Id, (u64, usize)> = HashMap::new();
        let mut ret: HashMap<u8, (u64, usize)> = HashMap::new();
        let mut tomb: HashMap<u8, (u64, usize)> = HashMap::new();
        for (i, u) in self.disk.iter().enumerate() {
            let (m, key, tree) = match &u.p { P::Leaf(id, _) => (0, Some(*id), id.tree), P::Ret(t, _) => (1, None, *t), P::Tomb(t, _) => (2, None, *t) };
            if !eligible(tree, u.birth) { continue; }
            let v = (u.birth, i);
            match m {
                0 => { let e = leaf.entry(key.unwrap()).or_insert(v); if v > *e { *e = v; } }
                1 => { let e = ret.entry(tree).or_insert(v); if v > *e { *e = v; } }
                _ => { let e = tomb.entry(tree).or_insert(v); if v > *e { *e = v; } }
            }
        }
        let retired: HashSet<Id> = ret.values().flat_map(|(_, i)| match &self.disk[*i].p { P::Ret(_, rs) => rs.iter().map(|r| r.0).collect::<Vec<_>>(), _ => vec![] }).collect();
        let dead: HashSet<(u64, u64)> = tomb.values().flat_map(|(_, i)| match &self.disk[*i].p { P::Tomb(_, ts) => ts.iter().map(|t| (t.0, t.1)).collect::<Vec<_>>(), _ => vec![] }).collect();
        let own_nums: HashSet<u64> = leaf.keys().filter(|id| id.tree == view).map(|id| id.num).collect();
        let mut got: HashMap<u64, (R, (u64, usize))> = HashMap::new();
        for (id, v) in &leaf {
            if retired.contains(id) { continue; }
            if num_takeover && view == C && id.tree == O && own_nums.contains(&id.num) { continue; }
            let P::Leaf(_, recs) = &self.disk[v.1].p else { unreachable!() };
            for r in recs {
                if dead.contains(&(r.ino, r.obirth)) { continue; }
                match got.get(&r.ino) { Some((_, k)) if *k >= *v => {} _ => { got.insert(r.ino, (*r, *v)); } }
            }
        }
        let h = &self.heads[self.hi(view)];
        let truth: BTreeMap<u64, R> = h.leaves.iter().flat_map(|l| l.recs.iter()).map(|r| (r.ino, *r)).collect();
        let r = got.iter().filter(|(ino, (rec, _))| truth.get(ino) != Some(rec)).count() as u64;
        let m = truth.keys().filter(|ino| Some(**ino) != h.queue && !got.contains_key(ino)).count() as u64;
        let q = h.queue.filter(|q| !got.contains_key(q)).map_or(0, |_| 1);
        (r, m, q)
    }
}

fn menu(w: &W, heads: &[u8]) -> Vec<Op> {
    let mut v = Vec::new();
    for &t in heads {
        v.push(Op::Create(t));
        let h = w.hi(t);
        let live: Vec<u64> = w.heads[h].leaves.iter().flat_map(|l| l.recs.iter()).filter(|r| r.nlink > 0).map(|r| r.ino).collect();
        for &x in &live { v.push(Op::Write(t, x)); }
        for &x in &live { v.push(Op::Unlink(t, x)); }
    }
    v
}

#[derive(Default, Clone)]
struct Agg { seqs: u64, checks: u64, r_seqs: u64, m_seqs: u64, q_seqs: u64, r_max: u64, m_max: u64, first: Option<String> }

pub fn run() {
    let readers: [(&str, Path, bool, bool); 8] = [
        ("C1_A", Path::C1, true, false), ("C1_B", Path::C1, false, false),
        ("C2lit_A", Path::C2lit, true, false), ("C2num_A", Path::C2lit, true, true), ("C2ret_A", Path::C2ret, true, false),
        ("TSZ_A", Path::Tsz, true, false), ("TSZ_B", Path::Tsz, false, false), ("C2num_B", Path::C2lit, false, true),
    ];
    let threads: usize = std::env::var("R3_THREADS").ok().and_then(|s| s.parse().ok()).unwrap_or(6).max(1);
    let jobs = std::sync::Arc::new(std::sync::Mutex::new(readers.to_vec()));
    let out = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let handles: Vec<_> = (0..threads).map(|_| {
        let (jobs, out) = (std::sync::Arc::clone(&jobs), std::sync::Arc::clone(&out));
        std::thread::spawn(move || loop {
            let job = jobs.lock().unwrap().pop();
            let Some((name, path, ancestry, num)) = job else { break };
            let r = one_reader(name, path, ancestry, num);
            out.lock().unwrap().push(r);
        })
    }).collect();
    for h in handles { h.join().expect("线程 panic"); }
    let mut aggs: BTreeMap<(String, u8), Agg> = BTreeMap::new();
    let mut prefixes = 0u64;
    for (p, a) in std::sync::Arc::try_unwrap(out).ok().expect("线程都收了").into_inner().unwrap() { prefixes = p; aggs.extend(a); }
    print_aggs(prefixes, &aggs);
}

fn one_reader(name: &'static str, path: Path, ancestry: bool, num: bool) -> (u64, BTreeMap<(String, u8), Agg>) {
    let mut aggs: BTreeMap<(String, u8), Agg> = BTreeMap::new();
    let prefixes;
    {
        let base = W::new(path);
        // 前缀：O 上 0–2 步全取。
        let mut pre: Vec<(Vec<Op>, W)> = vec![(vec![], base.clone())];
        let mut frontier = pre.clone();
        for _ in 0..2 {
            let mut next = Vec::new();
            for (ops, w) in &frontier {
                for op in menu(w, &[O]) {
                    let mut w2 = w.clone();
                    if w2.apply(op, &mut |_| {}) { let mut o2 = ops.clone(); o2.push(op); next.push((o2, w2)); }
                }
            }
            pre.extend(next.iter().cloned());
            frontier = next;
        }
        prefixes = pre.len() as u64;
        for (pre_ops, w0) in &pre {
            let mut w = w0.clone();
            w.fork();
            // 分叉之后：两个头上 0–3 步全取（深度优先），每次发布之后两个视图都判。
            let mut stack: Vec<(Vec<Op>, W)> = vec![(vec![], w)];
            while let Some((ops, w)) = stack.pop() {
                for view in [O, C] {
                    let a = aggs.entry((name.to_string(), view)).or_default();
                    a.seqs += 1;
                }
                if ops.len() == 3 { continue; }
                for op in menu(&w, &[O, C]) {
                    let mut w2 = w.clone();
                    let mut worst: HashMap<u8, (u64, u64, u64)> = HashMap::new();
                    let ok = w2.apply(op, &mut |x: &W| {
                        for view in [O, C] {
                            let (r, m, q) = x.judge(view, ancestry, num);
                            let e = worst.entry(view).or_insert((0, 0, 0));
                            e.0 = e.0.max(r); e.1 = e.1.max(m); e.2 = e.2.max(q);
                        }
                    });
                    if !ok { continue; }
                    let mut o2 = ops.clone();
                    o2.push(op);
                    for view in [O, C] {
                        let (r, m, q) = worst[&view];
                        let a = aggs.get_mut(&(name.to_string(), view)).unwrap();
                        a.checks += 1;
                        if r > 0 { a.r_seqs += 1; }
                        if m > 0 { a.m_seqs += 1; }
                        if q > 0 { a.q_seqs += 1; }
                        a.r_max = a.r_max.max(r);
                        a.m_max = a.m_max.max(m);
                        if (r > 0 || m > 0) && a.first.is_none() { a.first = Some(format!("prefix_on_O={pre_ops:?} after_fork={o2:?} r={r} m={m}").replace(' ', "")); }
                    }
                    stack.push((o2, w2));
                }
            }
        }
    }
    (prefixes, aggs)
}

fn print_aggs(prefixes: u64, aggs: &BTreeMap<(String, u8), Agg>) {
    println!("R3 name=clone prefixes={prefixes} leaf_cap={CAP} post_fork_steps_max=3");
    for ((name, view), a) in aggs {
        println!("R3 name=clone reader={name} view={} sequence_nodes={} op_checks={} r_ops={} m_ops={} q_ops={} r_max={} m_max={} first={}",
            if *view == O { "origin" } else { "clone" }, a.seqs, a.checks, a.r_seqs, a.m_seqs, a.q_seqs, a.r_max, a.m_max, a.first.clone().unwrap_or_else(|| "none".into()));
    }
}
