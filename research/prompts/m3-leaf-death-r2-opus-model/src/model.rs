//! m3-leaf-death-r2 攻方腿的模型：inode 叶容器（类型 2）、数据单元、退役记录容器（类型 3）三类单元落在槽里，
//! 发布、崩溃（在飞发布的单元写逐子集枚举）、丢根、管理员回退、unlink 分批、合并与借（N1 / N2 / 不借）都在这里；
//! 每次发布之后从盘上扫一遍、按几种读法重建最新那一版，与真值（逻辑状态）比。不依赖 crates/。
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::sync::Arc;

pub type Txg = u64;
pub type Inst = u32;
pub const PAYLOAD: u64 = 4096;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Ident { pub num: u64, pub birth: Txg }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rec { pub ino: u64, pub obirth: Txg, pub nlink: u32, pub size: u64 }

#[derive(Clone, Debug)]
pub struct Leaf { pub id: Ident, pub sep: u64, pub recs: Vec<Rec>, pub slot: Option<usize> }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetRec { pub who: Ident, pub death: Txg }

#[derive(Clone, Debug)]
pub struct RetC { pub id: Ident, pub recs: Vec<RetRec>, pub slot: Option<usize> }

#[derive(Clone, Copy, Debug)]
pub struct Anchor { pub content: u64, pub slot: Option<usize> }

#[derive(Clone, Debug)]
pub struct State {
    pub txg: Txg,
    pub inst: Inst,
    pub seq: u64,
    pub leaves: Vec<Leaf>,
    pub data: BTreeMap<u64, Vec<Anchor>>,
    pub rets: Vec<RetC>,
    pub queue: Option<u64>,
    pub watermark: u64,
    pub lineage: Vec<(Txg, Txg)>,
}

impl State {
    pub fn referenced_slots(&self) -> HashSet<usize> {
        let mut s = HashSet::new();
        for l in &self.leaves { s.insert(l.slot.expect("已发布的叶有槽")); }
        for a in self.data.values().flatten() { s.insert(a.slot.expect("已发布的数据单元有槽")); }
        for r in &self.rets { s.insert(r.slot.expect("已发布的退役容器有槽")); }
        s
    }
    pub fn record(&self, ino: u64) -> Option<Rec> {
        self.leaves.iter().find_map(|l| l.recs.iter().find(|r| r.ino == ino).copied())
    }
    pub fn leaf_of(&self, ino: u64) -> Option<usize> {
        self.leaves.iter().position(|l| l.recs.binary_search_by_key(&ino, |r| r.ino).is_ok())
    }
}

#[derive(Debug)]
pub enum Payload {
    Leaf { id: Ident, recs: Vec<Rec> },
    Data { ino: u64, obirth: Txg, anchor: u32, content: u64 },
    Ret { id: Ident, recs: Vec<RetRec> },
}

#[derive(Debug)]
pub struct Unit { pub birth: Txg, pub inst: Inst, pub payload: Payload }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Geometry { Wide, RingLowest, RingRandom }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SlotStatus { InUse, Pending(u64), Reusable }

#[derive(Clone)]
pub struct Alloc {
    pub geometry: Geometry,
    pub ring: u64,
    status: Vec<SlotStatus>,
    pending: VecDeque<(u64, usize)>,
    reusable: BTreeSet<usize>,
}

impl Alloc {
    pub fn new(geometry: Geometry, ring: u64) -> Self {
        Self { geometry, ring, status: Vec::new(), pending: VecDeque::new(), reusable: BTreeSet::new() }
    }
    pub fn alloc(&mut self, seq: u64, rng: &mut Rng) -> usize {
        while let Some(&(released, slot)) = self.pending.front() {
            if released + self.ring > seq { break; }
            self.pending.pop_front();
            if self.status[slot] == SlotStatus::Pending(released) {
                self.status[slot] = SlotStatus::Reusable;
                self.reusable.insert(slot);
            }
        }
        let reuse = match self.geometry {
            Geometry::Wide => None,
            Geometry::RingLowest => self.reusable.iter().next().copied(),
            Geometry::RingRandom => {
                if self.reusable.is_empty() { None } else {
                    let k = (rng.next() % self.reusable.len() as u64) as usize;
                    self.reusable.iter().nth(k).copied()
                }
            }
        };
        let slot = match reuse {
            Some(slot) => { self.reusable.remove(&slot); slot }
            None => { self.status.push(SlotStatus::InUse); self.status.len() - 1 }
        };
        self.status[slot] = SlotStatus::InUse;
        slot
    }
    /// 换成这一版引用的槽：引用到的一律 InUse（回退、恢复时把上一版放出去的槽收回来），InUse 而没被引用的放出去。
    pub fn adopt(&mut self, referenced: &HashSet<usize>, seq: u64) {
        for slot in 0..self.status.len() {
            let wanted = referenced.contains(&slot);
            match (self.status[slot], wanted) {
                (SlotStatus::InUse, false) => {
                    self.status[slot] = SlotStatus::Pending(seq);
                    if self.geometry != Geometry::Wide { self.pending.push_back((seq, slot)); }
                }
                (SlotStatus::Reusable, true) => { self.reusable.remove(&slot); self.status[slot] = SlotStatus::InUse; }
                (SlotStatus::Pending(_), true) => { self.status[slot] = SlotStatus::InUse; }
                _ => {}
            }
        }
    }
}

#[derive(Clone)]
pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    pub fn below(&mut self, n: u64) -> u64 { if n == 0 { 0 } else { self.next() % n } }
    pub fn chance(&mut self, per_mille: u64) -> bool { self.below(1000) < per_mille }
}

pub fn units_of(size: u64) -> u64 { size.div_ceil(PAYLOAD) }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BorrowArm { NoBorrow, N1, N2 }

#[derive(Clone, Debug)]
pub struct Config {
    pub name: String,
    pub seed: u64,
    pub geometry: Geometry,
    pub ring: u64,
    pub leaf_cap: usize,
    pub ret_cap: usize,
    pub borrow: BorrowArm,
    pub initial_inodes: u64,
    pub create_max: u64,
    pub actions: u32,
    /// 千分比：建、覆盖写、顺序写，其余是 unlink；回退、崩溃、丢根另掷。
    pub p_create: u64,
    pub p_write: u64,
    pub p_append: u64,
    pub p_rollback: u64,
    pub p_crash: u64,
    pub p_lost_root: u64,
    pub crash_k_max: u32,
    /// 管理员回退那次退役树取谁：false 随现行那一版（与墓碑树同），true 取 R_old 那一版（与 inode 树同）。
    pub ret_from_r_old: bool,
}

#[derive(Clone, Default, Debug)]
pub struct Cell { pub scans: u64, pub r_peak: u64, pub m_peak: u64, pub r_scans: u64, pub m_scans: u64, pub first: Option<String> }

#[derive(Clone, Default, Debug)]
pub struct Stats {
    pub cells: BTreeMap<(String, String), Cell>,
    pub counts: BTreeMap<String, u64>,
}
impl Stats {
    pub fn add(&mut self, key: &str, n: u64) { *self.counts.entry(key.to_string()).or_default() += n; }
    pub fn peak(&mut self, key: &str, n: u64) { let e = self.counts.entry(key.to_string()).or_default(); if n > *e { *e = n; } }
    pub fn merge(&mut self, other: &Stats) {
        for (k, c) in &other.cells {
            let e = self.cells.entry(k.clone()).or_default();
            e.scans += c.scans; e.r_scans += c.r_scans; e.m_scans += c.m_scans;
            e.r_peak = e.r_peak.max(c.r_peak); e.m_peak = e.m_peak.max(c.m_peak);
            if e.first.is_none() { e.first.clone_from(&c.first); }
        }
        for (k, v) in &other.counts {
            if k.ends_with("_peak") { self.peak(k, *v); } else { self.add(k, *v); }
        }
    }
    fn judge(&mut self, reader: &str, tree: &str, r: u64, m: u64, at: &str) {
        let e = self.cells.entry((reader.to_string(), tree.to_string())).or_default();
        e.scans += 1;
        if r > 0 { e.r_scans += 1; }
        if m > 0 { e.m_scans += 1; }
        e.r_peak = e.r_peak.max(r);
        e.m_peak = e.m_peak.max(m);
        if (r > 0 || m > 0) && e.first.is_none() { e.first = Some(format!("{at} r={r} m={m}")); }
    }
}

#[derive(Clone)]
pub struct World {
    pub cfg: Config,
    pub disk: Vec<Option<Arc<Unit>>>,
    pub alloc: Alloc,
    pub cur: State,
    pub line: Vec<State>,
    pub ring: VecDeque<(u64, Txg, bool)>,
    pub rows: BTreeMap<Inst, Txg>,
    pub seq: u64,
    pub next_txg: Txg,
    pub inst_now: Inst,
    pub max_inst: Inst,
    pub rng: Rng,
    pub content_counter: u64,
    pub stats: Stats,
    pub inject: bool,
    pub batch_k: usize,
    pub abandoned_objects: HashSet<(u64, Txg)>,
    pub structural_seqs: BTreeSet<u64>,
    pub history: Vec<String>,
    pub record_history: bool,
    pub force_crash: bool,
    pub dumped: bool,
}

pub enum Recovery { Crash, Lost { back: usize, jump: bool } }

impl World {
    pub fn new(cfg: Config) -> Self {
        let rng = Rng(cfg.seed ^ 0x5EED_0000_0000_0001);
        let geometry = cfg.geometry;
        let ring = cfg.ring;
        let empty = State { txg: 0, inst: 1, seq: 0, leaves: Vec::new(), data: BTreeMap::new(), rets: Vec::new(), queue: None, watermark: 1, lineage: Vec::new() };
        let mut w = World {
            cfg, disk: Vec::new(), alloc: Alloc::new(geometry, ring), cur: empty.clone(), line: vec![empty], ring: VecDeque::new(),
            rows: BTreeMap::new(), seq: 1, next_txg: 1, inst_now: 1, max_inst: 1, rng, content_counter: 0, stats: Stats::default(),
            inject: true, batch_k: 1, abandoned_objects: HashSet::new(), structural_seqs: BTreeSet::new(), history: Vec::new(), record_history: false, force_crash: false, dumped: false,
        };
        w.ring.push_back((0, 0, true));
        w
    }

    fn note(&mut self, s: String) { if self.record_history { self.history.push(s); } }

    fn dirty_count(next: &State) -> usize {
        next.leaves.iter().filter(|l| l.slot.is_none()).count()
            + next.data.values().flatten().filter(|a| a.slot.is_none()).count()
            + next.rets.iter().filter(|r| r.slot.is_none()).count()
    }

    /// 把 `next` 里没有槽的单元写出去；`mask` 给了就只写第 i 位为 1 的那几个（崩溃时在飞发布写出的子集）。
    fn write_dirty(&mut self, next: &mut State, mask: Option<u64>) {
        let (txg, inst, seq) = (next.txg, self.inst_now, self.seq);
        let mut i = 0u32;
        let take = |i: &mut u32| { let t = mask.map_or(true, |m| m >> *i & 1 == 1); *i += 1; t };
        let mut writes: Vec<(usize, Payload)> = Vec::new();
        let mut leaf_units = 0u64; let mut data_units = 0u64; let mut ret_units = 0u64;
        for leaf in &mut next.leaves {
            if leaf.slot.is_none() && take(&mut i) {
                let slot = self.alloc.alloc(seq, &mut self.rng);
                leaf.slot = Some(slot);
                writes.push((slot, Payload::Leaf { id: leaf.id, recs: leaf.recs.clone() }));
                leaf_units += 1;
            }
        }
        let births: HashMap<u64, Txg> = next.leaves.iter().flat_map(|l| l.recs.iter().map(|r| (r.ino, r.obirth))).collect();
        for (ino, anchors) in &mut next.data {
            for (k, a) in anchors.iter_mut().enumerate() {
                if a.slot.is_none() && take(&mut i) {
                    let slot = self.alloc.alloc(seq, &mut self.rng);
                    a.slot = Some(slot);
                    writes.push((slot, Payload::Data { ino: *ino, obirth: births[ino], anchor: k as u32, content: a.content }));
                    data_units += 1;
                }
            }
        }
        for r in &mut next.rets {
            if r.slot.is_none() && take(&mut i) {
                let slot = self.alloc.alloc(seq, &mut self.rng);
                r.slot = Some(slot);
                writes.push((slot, Payload::Ret { id: r.id, recs: r.recs.clone() }));
                ret_units += 1;
            }
        }
        for (slot, payload) in writes {
            if self.disk.len() <= slot { self.disk.resize_with(slot + 1, || None); }
            self.disk[slot] = Some(Arc::new(Unit { birth: txg, inst, payload }));
        }
        if mask.is_none() {
            self.stats.add("units_leaf", leaf_units);
            self.stats.add("units_data", data_units);
            self.stats.add("units_retirement", ret_units);
        }
    }

    /// 一次发布。注入打开时先掷崩溃：在飞发布的 k 个单元写逐子集枚举（每个子集各开一支：恢复、挂载、续做，每次发布之后都扫），
    /// 主线取全写出那一支往下走。不崩就整写、记根，再掷丢根。交回这次发布做没做成（崩了交回 false，调用方不再往下发）。
    pub fn publish(&mut self, mut next: State, what: &str, structural: bool) -> bool {
        next.txg = self.next_txg;
        next.inst = self.inst_now;
        next.seq = self.seq;
        let k = Self::dirty_count(&next) as u32;
        if k >= 1 && (self.force_crash || (self.inject && self.rng.chance(self.cfg.p_crash))) {
            self.force_crash = false;
            if k > self.cfg.crash_k_max {
                self.stats.add("crash_points_skipped_more_writes_than_k_max", 1);
            } else {
                self.note(format!("crash at txg {} inst {} during {what} k={k}", next.txg, self.inst_now));
                self.stats.add("crash_points", 1);
                self.stats.add("crash_states_enumerated", 1u64 << k);
                self.stats.peak("crash_k_peak", u64::from(k));
                let created: Vec<(u64, Txg)> = Self::created_objects(&self.cur, &next);
                for mask in 0..(1u64 << k) {
                    let mut branch = self.clone();
                    branch.inject = false;
                    branch.stats = Stats::default();
                    branch.record_history = false;
                    let mut n = next.clone();
                    branch.write_dirty(&mut n, Some(mask));
                    branch.seq += 1;
                    branch.recover(Recovery::Crash);
                    let s = branch.stats.clone();
                    self.stats.merge(&s);
                }
                let mut n = next.clone();
                self.write_dirty(&mut n, Some(u64::MAX));
                self.seq += 1;
                for o in created { self.abandoned_objects.insert(o); }
                self.recover(Recovery::Crash);
                return false;
            }
        }
        let created: Vec<(u64, Txg)> = Self::created_objects(&self.cur, &next);
        for o in &created {
            if self.abandoned_objects.contains(o) { self.stats.add("objects_reissued_with_an_abandoned_number_and_birth", 1); }
        }
        self.write_dirty(&mut next, None);
        self.alloc.adopt(&next.referenced_slots(), self.seq);
        if structural { self.structural_seqs.insert(self.seq); }
        self.note(format!("publish txg {} inst {} seq {} {what}", next.txg, next.inst, self.seq));
        self.ring.push_back((self.seq, next.txg, true));
        while self.ring.len() as u64 > self.cfg.ring { self.ring.pop_front(); }
        self.seq += 1;
        self.next_txg += 1;
        self.cur = next.clone();
        self.line.push(next);
        let keep = self.cfg.ring as usize + 8;
        if self.line.len() > keep { let cut = self.line.len() - keep; self.line.drain(..cut); }
        self.stats.add("publishes", 1);
        self.check();
        if self.inject && self.line.len() > 4 && self.rng.chance(self.cfg.p_lost_root) {
            let variant = self.rng.below(3);
            let (back, jump) = match variant { 0 => (1, false), 1 => (1, true), _ => (3, false) };
            let lost: Vec<(u64, Txg)> = (0..back).flat_map(|b| {
                let l = &self.line; Self::created_objects(&l[l.len() - 2 - b], &l[l.len() - 1 - b])
            }).collect();
            for o in lost { self.abandoned_objects.insert(o); }
            self.stats.add(match variant { 0 => "lost_root_one", 1 => "lost_root_one_readable_but_unusable", _ => "lost_root_three" }, 1);
            self.note(format!("lost root back={back} jump={jump}"));
            self.recover(Recovery::Lost { back, jump });
            return false;
        }
        true
    }

    fn created_objects(before: &State, after: &State) -> Vec<(u64, Txg)> {
        after.leaves.iter().flat_map(|l| l.recs.iter())
            .filter(|r| r.ino >= before.watermark)
            .map(|r| (r.ino, r.obirth)).collect()
    }

    pub fn recover(&mut self, how: Recovery) {
        let (back, jump) = match how { Recovery::Crash => (0, false), Recovery::Lost { back, jump } => (back, jump) };
        let sel_idx = self.line.len() - 1 - back;
        if !jump {
            let n = self.ring.len();
            for e in self.ring.iter_mut().skip(n - back) { e.2 = false; }
        }
        let sel = self.line[sel_idx].clone();
        self.line.truncate(sel_idx + 1);
        let new_inst = self.max_inst + 1;
        for inst in sel.inst..new_inst {
            self.rows.insert(inst, if inst == sel.inst { sel.txg } else { 0 });
        }
        self.inst_now = new_inst;
        self.max_inst = new_inst;
        self.next_txg = self.ring.iter().filter(|e| e.2).map(|e| e.1).max().unwrap_or(0) + 1;
        self.alloc.adopt(&sel.referenced_slots(), self.seq);
        self.verify_intact(&sel);
        self.cur = sel;
        self.stats.add("recoveries", 1);
        // 挂载：写行、暖机两次空发布，再把队列续做到空。
        let s = self.cur.clone();
        if !self.publish(s, "mount instance row", false) { return; }
        let s = self.cur.clone();
        if !self.publish(s, "mount warm-up", false) { return; }
        self.continue_queue();
    }

    fn verify_intact(&self, s: &State) {
        for l in &s.leaves {
            match self.disk[l.slot.unwrap()].as_deref() {
                Some(Unit { payload: Payload::Leaf { id, recs }, .. }) if *id == l.id && *recs == l.recs => {}
                other => panic!("模型错：回到的那一版的叶 {:?} 不在它的槽里：{other:?}", l.id),
            }
        }
        for (ino, anchors) in &s.data {
            for (k, a) in anchors.iter().enumerate() {
                match self.disk[a.slot.unwrap()].as_deref() {
                    Some(Unit { payload: Payload::Data { ino: i, anchor, content, .. }, .. }) if i == ino && *anchor == k as u32 && *content == a.content => {}
                    other => panic!("模型错：回到的那一版的数据单元 {ino}/{k} 不在它的槽里：{other:?}"),
                }
            }
        }
    }

    pub fn rollback(&mut self) -> bool {
        let n = self.line.len();
        let candidates: Vec<usize> = (0..n.saturating_sub(1))
            .filter(|&i| self.seq - self.line[i].seq + 2 <= self.cfg.ring && self.line[i].txg > 0)
            .collect();
        if candidates.is_empty() { return true; }
        let i = candidates[self.rng.below(candidates.len() as u64) as usize];
        self.rollback_to(i)
    }

    pub fn rollback_candidates(&self) -> Vec<usize> {
        let n = self.line.len();
        (0..n.saturating_sub(1)).filter(|&i| self.seq - self.line[i].seq + 2 <= self.cfg.ring && self.line[i].txg > 0).collect()
    }

    pub fn rollback_to(&mut self, i: usize) -> bool {
        let target = self.line[i].clone();
        let across = self.structural_seqs.range(target.seq + 1..).next().is_some();
        self.stats.add("rollbacks", 1);
        if across { self.stats.add("rollbacks_across_a_structural_publish", 1); }
        let mut next = target.clone();
        if !self.cfg.ret_from_r_old { next.rets = self.cur.rets.clone(); }
        next.watermark = self.cur.watermark.max(target.watermark);
        next.lineage = self.cur.lineage.clone();
        next.lineage.push((self.next_txg, target.txg));
        self.verify_intact(&target);
        self.line.truncate(i + 1);
        let structural: Vec<u64> = self.structural_seqs.range(target.seq + 1..).copied().collect();
        for s in structural { self.structural_seqs.remove(&s); }
        self.note(format!("rollback to txg {}", target.txg));
        if !self.publish(next, "admin rollback", false) { return false; }
        self.continue_queue();
        true
    }

    pub fn new_content(&mut self) -> u64 { self.content_counter += 1; self.content_counter }

    pub fn create(&mut self, count: u64) -> bool {
        let mut next = self.cur.clone();
        let txg = self.next_txg;
        let cap = self.cfg.leaf_cap;
        for _ in 0..count {
            let ino = next.watermark;
            next.watermark += 1;
            let rec = Rec { ino, obirth: txg, nlink: 1, size: 0 };
            match next.leaves.last_mut() {
                Some(last) if last.recs.len() < cap => { last.recs.push(rec); last.slot = None; }
                _ => next.leaves.push(Leaf { id: Ident { num: ino, birth: txg }, sep: ino, recs: vec![rec], slot: None }),
            }
        }
        self.publish(next, "create", false)
    }

    fn set_record(next: &mut State, rec: Rec) {
        let p = next.leaf_of(rec.ino).expect("记录在树里");
        let l = &mut next.leaves[p];
        let k = l.recs.binary_search_by_key(&rec.ino, |r| r.ino).unwrap();
        l.recs[k] = rec;
        l.slot = None;
    }

    /// 覆盖写：写 u 个单元、size 落在 u 个单元里；`same_size` 时 size 与单元数不变、每个单元换一份新内容（K3「size 不变而单元换了」）。
    pub fn overwrite(&mut self, ino: u64, units: u64, same_size: bool) -> bool {
        let mut next = self.cur.clone();
        let mut rec = next.record(ino).unwrap();
        let (u, size) = if same_size && rec.size > 0 { (units_of(rec.size), rec.size) } else {
            let size = units * PAYLOAD - self.rng.below(PAYLOAD);
            (units, size)
        };
        let anchors: Vec<Anchor> = (0..u).map(|_| Anchor { content: self.new_content(), slot: None }).collect();
        next.data.insert(ino, anchors);
        rec.size = size;
        Self::set_record(&mut next, rec);
        self.publish(next, if same_size { "overwrite same size" } else { "overwrite" }, false)
    }

    pub fn append(&mut self, ino: u64, more: u64) -> bool {
        let mut next = self.cur.clone();
        let mut rec = next.record(ino).unwrap();
        let u0 = units_of(rec.size);
        let anchors = next.data.entry(ino).or_default();
        assert_eq!(anchors.len() as u64, u0, "文件没有洞：单元数 = ⌈size ÷ 净荷⌉");
        for _ in 0..more { self.content_counter += 1; anchors.push(Anchor { content: self.content_counter, slot: None }); }
        rec.size = (u0 + more) * PAYLOAD;
        Self::set_record(&mut next, rec);
        self.publish(next, "append", false)
    }

    pub fn unlink(&mut self, ino: u64) -> bool {
        let mut next = self.cur.clone();
        let mut rec = next.record(ino).unwrap();
        rec.nlink = 0;
        Self::set_record(&mut next, rec);
        next.queue = Some(ino);
        self.batch_k = 1 + self.rng.below(3) as usize;
        if !self.publish(next, "unlink intent", false) { return false; }
        self.continue_queue()
    }

    /// 把现行那一版的队列走到空（会话里的续做、挂载续做、回退之后续做是同一条）。
    pub fn continue_queue(&mut self) -> bool {
        while let Some(ino) = self.cur.queue {
            let mut next = self.cur.clone();
            let anchors = next.data.get(&ino).map_or(0, Vec::len);
            if anchors > self.batch_k {
                let keep = anchors - self.batch_k;
                next.data.get_mut(&ino).unwrap().truncate(keep);
                let mut rec = next.record(ino).unwrap();
                rec.size = keep as u64 * PAYLOAD;
                Self::set_record(&mut next, rec);
                if !self.publish(next, "unlink batch", false) { return false; }
            } else {
                next.data.remove(&ino);
                next.queue = None;
                let structural = self.remove_record(&mut next, ino);
                if !self.publish(next, "unlink last batch", structural) { return false; }
            }
        }
        true
    }

    fn retire(&mut self, next: &mut State, who: Ident) {
        let death = self.next_txg;
        self.stats.add("retirement_records", 1);
        let cap = self.cfg.ret_cap;
        match next.rets.last_mut() {
            Some(open) if open.recs.len() < cap => { open.recs.push(RetRec { who, death }); open.slot = None; }
            _ => {
                let num = next.rets.iter().map(|r| r.id.num).max().unwrap_or(0) + 1;
                next.rets.push(RetC { id: Ident { num, birth: death }, recs: vec![RetRec { who, death }], slot: None });
            }
        }
    }

    fn moved_with_data(&mut self, next: &State, recs: &[Rec]) {
        let n = recs.iter().filter(|r| next.data.get(&r.ino).is_some_and(|a| !a.is_empty())).count() as u64;
        self.stats.add("records_with_data_moved_to_another_leaf", n);
    }

    /// unlink 最后一批：删那一条，再照 M2 / M3 均分借的写路径处理那一片。交回这次发布有没有身份消失或借（结构性的）。
    fn remove_record(&mut self, next: &mut State, ino: u64) -> bool {
        let cap = self.cfg.leaf_cap;
        let floor = cap / 2;
        let p = next.leaf_of(ino).expect("要删的记录在树里");
        let k = next.leaves[p].recs.binary_search_by_key(&ino, |r| r.ino).unwrap();
        next.leaves[p].recs.remove(k);
        next.leaves[p].slot = None;
        let len = |s: &State, i: usize| s.leaves[i].recs.len();
        if next.leaves[p].recs.is_empty() {
            let gone = next.leaves.remove(p);
            self.retire(next, gone.id);
            for l in next.leaves.iter_mut().skip(p) { l.slot = None; }
            self.stats.add("drops", 1);
            return true;
        }
        if p + 1 < next.leaves.len() && len(next, p) + len(next, p + 1) <= cap {
            let right = next.leaves.remove(p + 1);
            self.retire(next, right.id);
            self.moved_with_data(next, &right.recs);
            next.leaves[p].recs.extend(right.recs);
            for l in next.leaves.iter_mut().skip(p) { l.slot = None; }
            self.stats.add("absorbs", 1);
            return true;
        }
        if p > 0 && len(next, p - 1) + len(next, p) <= cap {
            let right = next.leaves.remove(p);
            self.retire(next, right.id);
            self.moved_with_data(next, &right.recs);
            next.leaves[p - 1].recs.extend(right.recs);
            for l in next.leaves.iter_mut().skip(p - 1) { l.slot = None; }
            self.stats.add("absorbs", 1);
            return true;
        }
        if self.cfg.borrow != BorrowArm::NoBorrow && len(next, p) < floor && p + 1 < next.leaves.len() {
            let n = (len(next, p + 1) - len(next, p)) / 2;
            if n >= 1 {
                let moved: Vec<Rec> = next.leaves[p + 1].recs.drain(..n).collect();
                self.moved_with_data(next, &moved);
                next.leaves[p].recs.extend(moved);
                next.leaves[p + 1].slot = None;
                let new_min = next.leaves[p + 1].recs[0].ino;
                match self.cfg.borrow {
                    BorrowArm::N1 => next.leaves[p + 1].sep = new_min,
                    BorrowArm::N2 => {
                        let old = next.leaves[p + 1].id;
                        self.retire(next, old);
                        next.leaves[p + 1].id = Ident { num: new_min, birth: self.next_txg };
                        next.leaves[p + 1].sep = new_min;
                    }
                    BorrowArm::NoBorrow => unreachable!(),
                }
                self.stats.add("borrows", 1);
                return true;
            }
        }
        false
    }


    /// trace 用：「统一滤法」读法第一次复活时，把复活的记录、它那一组被谁退役、那片退役容器在盘上的每一版（诞生代号、实例、
    /// 在不在谱系父链上、过不过 P2、两种择法各取了哪一版）逐条列出来。
    pub fn diagnose_uniform(&self, chain: &HashSet<Txg>) -> Vec<String> {
        let mut out = Vec::new();
        let truth: HashMap<u64, (Txg, u32, u64)> = self.cur.leaves.iter().flat_map(|l| l.recs.iter()).map(|r| (r.ino, (r.obirth, r.nlink, r.size))).collect();
        let mut leaf_versions: BTreeMap<Ident, Vec<(Txg, Inst, usize)>> = BTreeMap::new();
        let mut ret_versions: BTreeMap<Ident, Vec<(Txg, Inst, usize)>> = BTreeMap::new();
        for (slot, u) in self.disk.iter().enumerate() {
            let Some(u) = u.as_deref() else { continue };
            match &u.payload {
                Payload::Leaf { id, .. } => leaf_versions.entry(*id).or_default().push((u.birth, u.inst, slot)),
                Payload::Ret { id, .. } => ret_versions.entry(*id).or_default().push((u.birth, u.inst, slot)),
                Payload::Data { .. } => {}
            }
        }
        let referenced: HashSet<usize> = self.cur.referenced_slots();
        out.push(format!("  current version txg={} inst={} lineage={:?} rows={:?}", self.cur.txg, self.cur.inst, self.cur.lineage, self.rows));
        for (id, vs) in &leaf_versions {
            let live_here = self.cur.leaves.iter().any(|l| l.id == *id);
            for (b, i, slot) in vs {
                let recs = self.leaf_recs(*slot);
                let stale: Vec<u64> = recs.iter().filter(|r| !truth.contains_key(&r.ino)).map(|r| r.ino).collect();
                if stale.is_empty() || !chain.contains(b) || !self.published(self.disk[*slot].as_deref().unwrap()) { continue; }
                out.push(format!("  leaf {:?} version birth={b} inst={i} slot={slot} on_chain=true published=true live_in_truth={live_here} records_not_in_truth={stale:?}", id));
            }
        }
        for (id, vs) in &ret_versions {
            for (b, i, slot) in vs {
                let u = self.disk[*slot].as_deref().unwrap();
                let Payload::Ret { recs, .. } = &u.payload else { unreachable!() };
                out.push(format!("  retirement container {:?} version birth={b} inst={i} slot={slot} on_chain={} published={} referenced_by_the_current_root={} records={:?}",
                    id, chain.contains(b), self.published(u), referenced.contains(slot),
                    recs.iter().map(|r| format!("{}/{}@{}{}", r.who.num, r.who.birth, r.death, if chain.contains(&r.death) { "" } else { "(off)" })).collect::<Vec<_>>()));
            }
        }
        out
    }

    fn chain(&self) -> HashSet<Txg> {
        let parents: HashMap<Txg, Txg> = self.cur.lineage.iter().copied().collect();
        let mut chain = HashSet::new();
        let mut t = self.cur.txg;
        loop {
            chain.insert(t);
            if t == 0 { break; }
            t = parents.get(&t).copied().unwrap_or(t - 1);
        }
        chain
    }

    fn published(&self, u: &Unit) -> bool {
        if u.inst == self.cur.inst { return u.birth <= self.cur.txg; }
        self.rows.get(&u.inst).map_or(true, |&t| u.birth <= t)
    }

    fn leaf_recs(&self, slot: usize) -> &[Rec] {
        match &self.disk[slot].as_deref().unwrap().payload { Payload::Leaf { recs, .. } => recs, _ => unreachable!() }
    }

    /// 扫盘重建最新那一版，按读法与真值比。三种滤法：全（P2 + 谱系）、去掉 P2、去掉谱系；全滤法下三种退役读法。
    pub fn check(&mut self) {
        let chain = self.chain();
        let at = format!("seq={} txg={} inst={}", self.cur.seq, self.cur.txg, self.cur.inst);
        let truth_inode: HashMap<u64, (Txg, u32, u64)> = self.cur.leaves.iter().flat_map(|l| l.recs.iter())
            .map(|r| (r.ino, (r.obirth, r.nlink, r.size))).collect();
        let truth_data: HashMap<(u64, Txg, u32), u64> = self.cur.data.iter().flat_map(|(ino, a)| {
            let b = truth_inode[ino].0;
            a.iter().enumerate().map(move |(k, x)| ((*ino, b, k as u32), x.content))
        }).collect();
        let mut judged: Vec<(String, &'static str, u64, u64)> = Vec::new();
        let mut dup_peak = 0u64;
        for (fname, p2, lin) in [("", true, true), ("_noP2", false, true), ("_noLin", true, false)] {
            let mut leaf_g: HashMap<Ident, (Txg, Inst, usize)> = HashMap::new();
            let mut data_g: HashMap<(u64, Txg, u32), (Txg, Inst, u64)> = HashMap::new();
            // 退役容器两种择版本：「随现行」的树只按 P2 择（不按谱系滤版本，K2.3）；「统一」按与叶同一套滤法。
            let mut ret_carried: HashMap<Ident, (Txg, Inst, usize)> = HashMap::new();
            let mut ret_uniform: HashMap<Ident, (Txg, Inst, usize)> = HashMap::new();
            for (slot, u) in self.disk.iter().enumerate() {
                let Some(u) = u.as_deref() else { continue };
                if p2 && !self.published(u) { continue; }
                let on_chain = chain.contains(&u.birth);
                let key = (u.birth, u.inst);
                match &u.payload {
                    Payload::Ret { id, .. } => {
                        let e = ret_carried.entry(*id).or_insert((u.birth, u.inst, slot)); if key > (e.0, e.1) { *e = (u.birth, u.inst, slot); }
                        if !lin || on_chain {
                            let e = ret_uniform.entry(*id).or_insert((u.birth, u.inst, slot)); if key > (e.0, e.1) { *e = (u.birth, u.inst, slot); }
                        }
                    }
                    _ if lin && !on_chain => {}
                    Payload::Leaf { id, .. } => { let e = leaf_g.entry(*id).or_insert((u.birth, u.inst, slot)); if key > (e.0, e.1) { *e = (u.birth, u.inst, slot); } }
                    Payload::Data { ino, obirth, anchor, content } => {
                        let e = data_g.entry((*ino, *obirth, *anchor)).or_insert((u.birth, u.inst, *content));
                        if key > (e.0, e.1) { *e = (u.birth, u.inst, *content); }
                    }
                }
            }
            // (读法名, 退役：None 不读 / Some((按死亡 txg 滤, 退役容器随现行择)), 数据读法)
            type Reader<'a> = (&'a str, Option<(bool, bool)>, &'a [&'static str]);
            let readers: &[Reader] = if fname.is_empty() {
                &[("S", None, &["data_Z", "data_S"]), ("C1n", Some((false, true)), &["data_Z"]), ("C1n_uniform", Some((false, false)), &["data_Z"]),
                  ("C1t_uniform", Some((true, false)), &["data_Z"]), ("C1t", Some((true, true)), &["data_Z", "data_RG", "data_S"])]
            } else { &[("C1t", Some((true, true)), &["data_Z"])] };
            for (rname, ret, data_readers) in readers {
                let name = format!("{rname}{fname}");
                let mut retired: HashSet<Ident> = HashSet::new();
                if let Some((by_lineage, carried)) = ret {
                    let groups = if *carried { &ret_carried } else { &ret_uniform };
                    for (_, _, slot) in groups.values() {
                        if let Payload::Ret { recs, .. } = &self.disk[*slot].as_deref().unwrap().payload {
                            for r in recs { if !*by_lineage || chain.contains(&r.death) { retired.insert(r.who); } }
                        }
                    }
                }
                // 同号几条取供出它的叶版本里 (诞生代号, 实例, 槽) 最大的那一条：平局（只在消融读法里出现）也按槽号定，输出不随哈希次序变。
                let mut inode: HashMap<u64, (Rec, Txg)> = HashMap::new();
                let mut chosen: HashMap<u64, (Txg, Inst, usize)> = HashMap::new();
                let mut dups = 0u64;
                for (id, (birth, inst, slot)) in &leaf_g {
                    if retired.contains(id) { continue; }
                    for rec in self.leaf_recs(*slot) {
                        let key = (*birth, *inst, *slot);
                        match chosen.get(&rec.ino) {
                            Some(k) => { dups += 1; if key > *k { chosen.insert(rec.ino, key); inode.insert(rec.ino, (*rec, *birth)); } }
                            None => { chosen.insert(rec.ino, key); inode.insert(rec.ino, (*rec, *birth)); }
                        }
                    }
                }
                if name == "C1t" { dup_peak = dup_peak.max(dups); }
                let r = inode.iter().filter(|(ino, (rec, _))| truth_inode.get(ino) != Some(&(rec.obirth, rec.nlink, rec.size))).count() as u64;
                let m = truth_inode.keys().filter(|ino| !inode.contains_key(ino)).count() as u64;
                judged.push((name.clone(), "inode", r, m));
                for dr in *data_readers {
                    let kept: HashMap<(u64, Txg, u32), u64> = data_g.iter().filter(|((ino, ob, anchor), _)| match *dr {
                        "data_S" => true,
                        "data_RG" => inode.get(ino).is_some_and(|(rec, _)| rec.obirth == *ob),
                        _ => inode.get(ino).is_some_and(|(rec, _)| rec.obirth == *ob && u64::from(*anchor) < units_of(rec.size)),
                    }).map(|(k, v)| (*k, v.2)).collect();
                    let r = kept.iter().filter(|(k, c)| truth_data.get(k) != Some(c)).count() as u64;
                    let m = truth_data.keys().filter(|k| !kept.contains_key(k)).count() as u64;
                    judged.push((name.clone(), dr, r, m));
                }
            }
        }
        if self.record_history && !self.dumped && judged.iter().any(|(n, t, r, _)| n == "C1t_uniform" && *t == "inode" && *r > 0) {
            self.dumped = true;
            let d = self.diagnose_uniform(&chain);
            self.history.push(format!("FIRST C1t_uniform inode r>0 at {at}"));
            self.history.extend(d);
        }
        for (name, tree, r, m) in judged { self.stats.judge(&name, tree, r, m, &at); }
        self.stats.peak("duplicate_inode_numbers_across_collected_groups_C1t_peak", dup_peak);
        self.separators();
    }

    /// K4：这一版存着的分隔 key 与两种「从叶重建」的取法（容器号、孩子现在的最小 key）逐片比；I-9.12 与按分隔 key 路由各判一遍。
    fn separators(&mut self) {
        let leaves = &self.cur.leaves;
        if leaves.is_empty() { return; }
        let route = |seps: &[u64], ino: u64| seps.iter().rposition(|&s| s <= ino);
        let stored: Vec<u64> = leaves.iter().map(|l| l.sep).collect();
        let by_number: Vec<u64> = leaves.iter().map(|l| l.id.num).collect();
        let by_min: Vec<u64> = leaves.iter().map(|l| l.recs[0].ino).collect();
        let mut out: Vec<(&str, u64)> = Vec::new();
        for (name, seps) in [("stored", &stored), ("rebuilt_by_container_number", &by_number), ("rebuilt_by_current_smallest_key", &by_min)] {
            let mut i912 = 0u64;
            let mut misrouted = 0u64;
            for (i, l) in leaves.iter().enumerate() {
                if seps[i] > l.recs[0].ino { i912 += 1; }
                if i > 0 && (seps[i] <= leaves[i - 1].recs.last().unwrap().ino || seps[i] <= seps[i - 1]) { i912 += 1; }
                misrouted += l.recs.iter().filter(|r| route(seps, r.ino) != Some(i)).count() as u64;
            }
            out.push((name, i912));
            out.push((name, misrouted));
        }
        let differs_from_min = stored.iter().zip(&by_min).filter(|(a, b)| a != b).count() as u64;
        let differs_from_number = stored.iter().zip(&by_number).filter(|(a, b)| a != b).count() as u64;
        let old_third_clause_false = (1..leaves.len()).filter(|&i| leaves[i - 1].recs.last().unwrap().ino >= leaves[i].id.num).count() as u64;
        let names = ["stored_i912_violations", "stored_misrouted_records", "by_number_i912_violations", "by_number_misrouted_records",
                     "by_min_i912_violations", "by_min_misrouted_records"];
        for (k, (_, v)) in out.iter().enumerate() {
            self.stats.peak(&format!("sep_{}_peak", names[k]), *v);
            if *v > 0 { self.stats.add(&format!("sep_versions_with_{}", names[k]), 1); }
        }
        if differs_from_min > 0 { self.stats.add("sep_versions_stored_differs_from_current_smallest_key", 1); }
        let neither = stored.iter().zip(&by_min).zip(&by_number).filter(|((s, m), n)| s != m && s != n).count() as u64;
        if neither > 0 { self.stats.add("sep_versions_stored_matches_neither_rebuild_rule", 1); }
        if differs_from_number > 0 { self.stats.add("sep_versions_stored_differs_from_container_number", 1); }
        if old_third_clause_false > 0 { self.stats.add("versions_with_i94_third_clause_by_container_number_false", 1); }
        self.stats.add("versions_judged", 1);
    }
}
