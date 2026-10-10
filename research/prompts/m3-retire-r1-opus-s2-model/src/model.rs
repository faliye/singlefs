//! m3-retire-r1 攻方腿第二次抽样（opus-s2）的模型。从 m3-leaf-death-r3 攻方 s2 模型
//! （`research/prompts/m3-leaf-death-r3-opus-s2-model/src/model.rs`）拷起改：只留 inode 叶容器（类型 2）与退役容器（类型 3）两类单元；
//! 写路径照 E176 的 M3（删空就摘、并到没有一对装得下、均分借、借完查右邻那一对；N1：借不换身份），叶只在内容变了时重写；
//! 管理员回退的目标放开到根环里全部有效根（含被前一次回退抛弃的那一段，D16 已定项 1「回退候选集」只要求按实例表有效 ∧ txg ≥ F）；
//! 每次发布之后：扫盘按几种退役读法重建（K2/K3），对这一版判 K5 的几条候选判据（合法镜像上不许红），记 K4 的每次发布退役条数。
//! 不依赖 crates/。
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::sync::Arc;

pub type Txg = u64;
pub type Inst = u32;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Ident { pub num: u64, pub birth: Txg }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rec { pub ino: u64, pub obirth: Txg, pub nlink: u32 }

/// `written`：这片叶最后一版写出时那次发布的 txg（「死亡 txg = 最后一版的诞生代号」那条对照臂要它）。
#[derive(Clone, Debug)]
pub struct Leaf { pub id: Ident, pub sep: u64, pub recs: Vec<Rec>, pub slot: Option<usize>, pub written: Txg }

/// 一条退役记录：被退役的身份、死亡 txg（摘掉 / 吸收那次发布的 checkpoint_txg）、被退役那片叶最后一版的诞生代号（只供对照臂读）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetRec { pub who: Ident, pub death: Txg, pub last_write: Txg }

#[derive(Clone, Debug)]
pub struct RetC { pub id: Ident, pub recs: Vec<RetRec>, pub slot: Option<usize> }

#[derive(Clone, Debug)]
pub struct State {
    pub txg: Txg,
    pub inst: Inst,
    pub seq: u64,
    pub leaves: Vec<Leaf>,
    pub rets: Vec<RetC>,
    pub queue: Option<u64>,
    pub watermark: u64,
    pub lineage: Vec<(Txg, Txg)>,
}

impl State {
    pub fn referenced_slots(&self) -> HashSet<usize> {
        let mut s = HashSet::new();
        for l in &self.leaves { s.insert(l.slot.expect("已发布的叶有槽")); }
        for r in &self.rets { s.insert(r.slot.expect("已发布的退役容器有槽")); }
        s
    }
    pub fn leaf_of(&self, ino: u64) -> Option<usize> {
        self.leaves.iter().position(|l| l.recs.binary_search_by_key(&ino, |r| r.ino).is_ok())
    }
    pub fn record(&self, ino: u64) -> Option<Rec> {
        self.leaf_of(ino).map(|p| { let l = &self.leaves[p]; l.recs[l.recs.binary_search_by_key(&ino, |r| r.ino).unwrap()] })
    }
    pub fn ret_records(&self) -> Vec<RetRec> {
        let mut cs: Vec<&RetC> = self.rets.iter().collect();
        cs.sort_by_key(|c| (c.id.num, c.id.birth));
        cs.iter().flat_map(|c| c.recs.iter().copied()).collect()
    }
    /// 这一版的谱系父链（I-9.18 的定义：有谱系记录 (T_rb, T_old) 的 T_rb 的父是 T_old，其余 txg 的父是 txg − 1）。
    pub fn chain(&self) -> HashSet<Txg> {
        let parents: HashMap<Txg, Txg> = self.lineage.iter().copied().collect();
        let mut chain = HashSet::new();
        let mut t = self.txg;
        loop {
            chain.insert(t);
            if t == 0 { break; }
            t = parents.get(&t).copied().unwrap_or(t - 1);
        }
        chain
    }
}

#[derive(Debug)]
pub enum Payload { Leaf { id: Ident, recs: Vec<Rec> }, Ret { id: Ident, recs: Vec<RetRec> } }

#[derive(Debug)]
pub struct Unit { pub birth: Txg, pub inst: Inst, pub payload: Payload }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Geometry { Wide, RingLowest, RingRandom }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SlotStatus { InUse, Pending(u64), Reusable }

#[derive(Clone)]
pub struct Alloc { pub geometry: Geometry, pub ring: u64, status: Vec<SlotStatus>, pending: VecDeque<(u64, usize)>, reusable: BTreeSet<usize> }

impl Alloc {
    pub fn new(geometry: Geometry, ring: u64) -> Self {
        Self { geometry, ring, status: Vec::new(), pending: VecDeque::new(), reusable: BTreeSet::new() }
    }
    pub fn alloc(&mut self, seq: u64, rng: &mut Rng) -> usize {
        while let Some(&(released, slot)) = self.pending.front() {
            if released + self.ring > seq { break; }
            self.pending.pop_front();
            if self.status[slot] == SlotStatus::Pending(released) { self.status[slot] = SlotStatus::Reusable; self.reusable.insert(slot); }
        }
        let reuse = match self.geometry {
            Geometry::Wide => None,
            Geometry::RingLowest => self.reusable.iter().next().copied(),
            Geometry::RingRandom => if self.reusable.is_empty() { None } else {
                let k = (rng.next() % self.reusable.len() as u64) as usize; self.reusable.iter().nth(k).copied()
            },
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
            match (self.status[slot], referenced.contains(&slot)) {
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

#[derive(Clone, Debug)]
pub struct Config {
    pub name: String,
    pub seed: u64,
    pub geometry: Geometry,
    pub ring: u64,
    pub leaf_cap: usize,
    pub ret_cap: usize,
    pub initial_inodes: u64,
    pub create_max: u64,
    pub actions: u32,
    /// 千分比：建；其余是 unlink。回退、崩溃、丢根另掷。
    pub p_create: u64,
    pub p_rollback: u64,
    pub p_crash: u64,
    pub p_lost_root: u64,
    pub crash_k_max: u32,
    /// 回退时千分之几只在「不在现行那一版谱系父链上」的候选里挑（有这种候选时）：把「回退到被前一次回退抛弃的那一段」挑出来多走几次。
    pub p_rollback_off_chain: u64,
    /// 前多少次用户动作照 M1（`crates/` 今天：只删空即摘）写，之后换 M3（K4 的升级那一形）；0 = 一直 M3。
    pub m1_actions: u32,
}

#[derive(Clone, Default, Debug)]
pub struct Cell { pub scans: u64, pub r_peak: u64, pub m_peak: u64, pub r_scans: u64, pub m_scans: u64, pub first: Option<String> }

#[derive(Clone, Default, Debug)]
pub struct Stats { pub cells: BTreeMap<(String, String), Cell>, pub counts: BTreeMap<String, u64> }
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
        for (k, v) in &other.counts { if k.ends_with("_peak") { self.peak(k, *v); } else { self.add(k, *v); } }
    }
    pub fn judge(&mut self, reader: &str, tree: &str, r: u64, m: u64, at: &str) {
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
    /// 已发布、没被崩溃恢复抛弃的版本，按发布先后（不按谱系）：丢根恢复按先后退，回退候选从这里挑。
    pub line: Vec<State>,
    pub ring: VecDeque<(u64, Txg, bool, Inst)>,
    pub rows: BTreeMap<Inst, Txg>,
    pub seq: u64,
    pub next_txg: Txg,
    pub inst_now: Inst,
    pub max_inst: Inst,
    pub rng: Rng,
    pub stats: Stats,
    pub inject: bool,
    pub m3: bool,
    pub history: Vec<String>,
    pub record_history: bool,
}

pub enum Recovery { Crash, Lost { back: usize, jump: bool } }

impl World {
    pub fn new(cfg: Config) -> Self {
        let rng = Rng(cfg.seed ^ 0x5EED_0000_0000_0001);
        let (geometry, ring) = (cfg.geometry, cfg.ring);
        let m3 = cfg.m1_actions == 0;
        let empty = State { txg: 0, inst: 1, seq: 0, leaves: Vec::new(), rets: Vec::new(), queue: None, watermark: 1, lineage: Vec::new() };
        let mut w = World {
            cfg, disk: Vec::new(), alloc: Alloc::new(geometry, ring), cur: empty.clone(), line: vec![empty], ring: VecDeque::new(),
            rows: BTreeMap::new(), seq: 1, next_txg: 1, inst_now: 1, max_inst: 1, rng, stats: Stats::default(),
            inject: true, m3, history: Vec::new(), record_history: false,
        };
        w.ring.push_back((0, 0, true, 1));
        w
    }

    pub fn note(&mut self, s: String) { if self.record_history { self.history.push(s); } }

    fn dirty_count(next: &State) -> usize {
        next.leaves.iter().filter(|l| l.slot.is_none()).count() + next.rets.iter().filter(|r| r.slot.is_none()).count()
    }

    /// 把 `next` 里没有槽的单元写出去；`mask` 给了就只写第 i 位为 1 的那几个（崩溃时在飞发布写出的子集）。
    fn write_dirty(&mut self, next: &mut State, mask: Option<u64>) {
        let (txg, inst, seq) = (next.txg, self.inst_now, self.seq);
        let mut i = 0u32;
        let take = |i: &mut u32| { let t = mask.map_or(true, |m| m >> *i & 1 == 1); *i += 1; t };
        let mut writes: Vec<(usize, Payload)> = Vec::new();
        for leaf in &mut next.leaves {
            if leaf.slot.is_none() && take(&mut i) {
                let slot = self.alloc.alloc(seq, &mut self.rng);
                leaf.slot = Some(slot);
                leaf.written = txg;
                writes.push((slot, Payload::Leaf { id: leaf.id, recs: leaf.recs.clone() }));
            }
        }
        for r in &mut next.rets {
            if r.slot.is_none() && take(&mut i) {
                let slot = self.alloc.alloc(seq, &mut self.rng);
                r.slot = Some(slot);
                writes.push((slot, Payload::Ret { id: r.id, recs: r.recs.clone() }));
            }
        }
        for (slot, payload) in writes {
            if self.disk.len() <= slot { self.disk.resize_with(slot + 1, || None); }
            self.disk[slot] = Some(Arc::new(Unit { birth: txg, inst, payload }));
        }
    }

    fn ret_record_count(s: &State) -> u64 { s.rets.iter().map(|c| c.recs.len() as u64).sum() }

    /// 一次发布。注入打开时先掷崩溃：在飞发布的 k 个单元写逐子集枚举（每个子集各开一支：恢复、挂载、续做，每次发布之后都扫），
    /// 主线取全写出那一支往下走。不崩就整写、记根，再掷丢根。交回这次发布做没做成。
    pub fn publish(&mut self, mut next: State, what: &str) -> bool {
        next.txg = self.next_txg;
        next.inst = self.inst_now;
        next.seq = self.seq;
        let k = Self::dirty_count(&next) as u32;
        if k >= 1 && self.inject && self.rng.chance(self.cfg.p_crash) {
            if k > self.cfg.crash_k_max {
                self.stats.add("crash_points_skipped_more_writes_than_k_max", 1);
            } else {
                self.note(format!("crash at txg {} inst {} during {what} k={k}", next.txg, self.inst_now));
                self.stats.add("crash_points", 1);
                self.stats.add("crash_states_enumerated", 1u64 << k);
                self.stats.peak("crash_k_peak", u64::from(k));
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
                self.recover(Recovery::Crash);
                return false;
            }
        }
        let new_records = Self::ret_record_count(&next).saturating_sub(Self::ret_record_count(&self.cur));
        let new_containers = next.rets.iter().filter(|r| r.slot.is_none()).count() as u64;
        self.stats.peak("retirement_records_in_one_publish_peak", new_records);
        self.stats.peak("retirement_containers_written_in_one_publish_peak", new_containers);
        if new_records > 0 { self.stats.add(&format!("publishes_with_{new_records}_retirement_records"), 1); }
        self.write_dirty(&mut next, None);
        self.alloc.adopt(&next.referenced_slots(), self.seq);
        self.note(format!("publish txg {} inst {} seq {} {what} leaves={:?} ret_records={}", next.txg, next.inst, self.seq,
            next.leaves.iter().map(|l| (l.id.num, l.id.birth, l.recs.len())).collect::<Vec<_>>(), Self::ret_record_count(&next)));
        self.ring.push_back((self.seq, next.txg, true, self.inst_now));
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
            self.stats.add(match variant { 0 => "lost_root_one", 1 => "lost_root_one_readable_but_unusable", _ => "lost_root_three" }, 1);
            self.note(format!("lost root back={back} jump={jump}"));
            self.recover(Recovery::Lost { back, jump });
            return false;
        }
        true
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
        for inst in sel.inst..new_inst { self.rows.insert(inst, if inst == sel.inst { sel.txg } else { 0 }); }
        self.inst_now = new_inst;
        self.max_inst = new_inst;
        self.next_txg = self.ring.iter().filter(|e| e.2).map(|e| e.1).max().unwrap_or(0) + 1;
        self.alloc.adopt(&sel.referenced_slots(), self.seq);
        self.verify_intact(&sel);
        self.cur = sel;
        self.stats.add("recoveries", 1);
        // 挂载：写行、暖机两次空发布，再把队列续做到空。
        let s = self.cur.clone();
        if !self.publish(s, "mount instance row") { return; }
        let s = self.cur.clone();
        if !self.publish(s, "mount warm-up") { return; }
        self.continue_queue();
    }

    fn verify_intact(&self, s: &State) {
        for l in &s.leaves {
            match self.disk[l.slot.unwrap()].as_deref() {
                Some(Unit { payload: Payload::Leaf { id, recs }, .. }) if *id == l.id && *recs == l.recs => {}
                other => panic!("模型错：回到的那一版的叶 {:?} 不在它的槽里：{other:?}", l.id),
            }
        }
    }

    pub fn published(&self, u: &Unit) -> bool {
        if u.inst == self.cur.inst { return u.birth <= self.cur.txg; }
        self.rows.get(&u.inst).map_or(true, |&t| u.birth <= t)
    }

    /// 回退候选：根环里（按 seq 还在环里）、txg > 0、不是现行那一版的已发布版本，不论在不在现行那一版的谱系父链上。
    pub fn rollback_candidates(&self) -> Vec<usize> {
        let n = self.line.len();
        (0..n.saturating_sub(1)).filter(|&i| self.seq - self.line[i].seq + 2 <= self.cfg.ring && self.line[i].txg > 0).collect()
    }

    pub fn rollback(&mut self) -> bool {
        let all = self.rollback_candidates();
        if all.is_empty() { return true; }
        let chain = self.cur.chain();
        let off: Vec<usize> = all.iter().copied().filter(|&i| !chain.contains(&self.line[i].txg)).collect();
        let pick = if !off.is_empty() && self.rng.chance(self.cfg.p_rollback_off_chain) {
            self.stats.add("rollbacks_to_a_root_off_the_current_chain", 1);
            off[self.rng.below(off.len() as u64) as usize]
        } else { all[self.rng.below(all.len() as u64) as usize] };
        self.rollback_to(pick)
    }

    /// 管理员回退（D23 已定项 14）：新根的 inode 树取目标那一版；退役树随现行那一版；追加谱系记录 (T_rb, T_old)；水位取 max。
    pub fn rollback_to(&mut self, i: usize) -> bool {
        let target = self.line[i].clone();
        if !self.cur.chain().contains(&target.txg) { self.stats.add("rollbacks_off_chain_taken", 1); }
        self.stats.add("rollbacks", 1);
        let mut next = target.clone();
        next.rets = self.cur.rets.clone();
        next.watermark = self.cur.watermark.max(target.watermark);
        next.lineage = self.cur.lineage.clone();
        next.lineage.push((self.next_txg, target.txg));
        self.verify_intact(&target);
        self.note(format!("rollback to txg {}", target.txg));
        if !self.publish(next, "admin rollback") { return false; }
        self.continue_queue()
    }

    /// 不改任何东西的发布 n 次。
    pub fn idle(&mut self, n: u32) -> bool {
        for _ in 0..n { let s = self.cur.clone(); if !self.publish(s, "idle") { return false; } }
        true
    }

    /// 建 `count` 个空 inode：号从水位起，落在最右的叶，满了在末尾分裂（右半容器号 = 那条新记录的号、出生代 = 这次发布）。
    pub fn create(&mut self, count: u64) -> bool {
        let mut next = self.cur.clone();
        let txg = self.next_txg;
        let cap = self.cfg.leaf_cap;
        for _ in 0..count {
            let ino = next.watermark;
            next.watermark += 1;
            let rec = Rec { ino, obirth: txg, nlink: 1 };
            match next.leaves.last_mut() {
                Some(last) if last.recs.len() < cap => { last.recs.push(rec); last.slot = None; }
                _ => next.leaves.push(Leaf { id: Ident { num: ino, birth: txg }, sep: ino, recs: vec![rec], slot: None, written: 0 }),
            }
        }
        self.publish(next, "create")
    }

    /// unlink：意图那次（nlink = 0、进队列），再续做到空（没有数据单元，意图之后直接是最后一批：删记录，照 M1 / M3 处理那一片）。
    pub fn unlink(&mut self, ino: u64) -> bool {
        let mut next = self.cur.clone();
        let p = next.leaf_of(ino).expect("unlink 的号在树里");
        let k = next.leaves[p].recs.binary_search_by_key(&ino, |r| r.ino).unwrap();
        next.leaves[p].recs[k].nlink = 0;
        next.leaves[p].slot = None;
        next.queue = Some(ino);
        if !self.publish(next, "unlink intent") { return false; }
        self.continue_queue()
    }

    pub fn continue_queue(&mut self) -> bool {
        while let Some(ino) = self.cur.queue {
            let mut next = self.cur.clone();
            next.queue = None;
            self.remove_record(&mut next, ino);
            if !self.publish(next, "unlink last batch") { return false; }
        }
        true
    }

    /// 给 `gone` 的身份追加一条退役记录（照墓碑树：开放容器满了才关，下一个容器号 = 树里最大号 + 1）。死亡 txg = 这次发布。
    fn retire(&mut self, next: &mut State, gone: &Leaf) {
        let death = self.next_txg;
        if next.ret_records().iter().any(|r| r.who == gone.id) { self.stats.add("retirements_of_an_identity_already_in_the_carried_tree", 1); }
        self.stats.add("retirement_records", 1);
        let rec = RetRec { who: gone.id, death, last_write: gone.written };
        let cap = self.cfg.ret_cap;
        match next.rets.iter_mut().max_by_key(|c| c.id.num) {
            Some(open) if open.recs.len() < cap => { open.recs.push(rec); open.slot = None; }
            _ => {
                let num = next.rets.iter().map(|r| r.id.num).max().unwrap_or(0) + 1;
                next.rets.push(RetC { id: Ident { num, birth: death }, recs: vec![rec], slot: None });
            }
        }
    }

    fn pos_of(next: &State, id: Ident) -> usize { next.leaves.iter().position(|l| l.id == id).expect("身份还在树里") }

    /// 左片吸收它右边那片：左片保留身份，右片摘掉、给它的身份写一条退役记录。
    fn absorb(&mut self, next: &mut State, left: usize) {
        let right = next.leaves.remove(left + 1);
        self.retire(next, &right);
        next.leaves[left].recs.extend(right.recs.iter().copied());
        next.leaves[left].slot = None;
        self.stats.add("absorbs", 1);
    }

    /// E176 `merge_until_no_pair_fits`（先查右邻那一对）：和 ≤ 容量就并，并完从幸存那片接着查，直到没有一对装得下。
    fn merge_until(&mut self, next: &mut State, start: Ident) -> bool {
        let cap = self.cfg.leaf_cap;
        let mut current = start;
        let mut merged = false;
        loop {
            let pos = Self::pos_of(next, current);
            let n = next.leaves.len();
            let len = |s: &State, i: usize| s.leaves[i].recs.len();
            if pos + 1 < n && len(next, pos) + len(next, pos + 1) <= cap { self.absorb(next, pos); }
            else if pos > 0 && len(next, pos - 1) + len(next, pos) <= cap { self.absorb(next, pos - 1); current = next.leaves[pos - 1].id; }
            else { return merged; }
            merged = true;
        }
    }

    /// E176 M3 的借（N1）：记录数 < 下限且有右邻时，借右邻最小的 ⌊(|R| − |X|) ÷ 2⌋ 条，两片都保留身份、不写退役；
    /// 右邻的分隔 key 改成它这一版的最小 key；借完让右邻与它右邻那一对照并的规则查。
    fn borrow(&mut self, next: &mut State, id: Ident) {
        let floor = self.cfg.leaf_cap.div_ceil(2);
        let pos = Self::pos_of(next, id);
        if pos + 1 >= next.leaves.len() || next.leaves[pos].recs.len() >= floor { return; }
        let (have, lender) = (next.leaves[pos].recs.len(), next.leaves[pos + 1].recs.len());
        let n = lender.saturating_sub(have) / 2;
        if n < 1 || n >= lender { self.stats.add("borrows_skipped_lender_too_small", 1); return; }
        let moved: Vec<Rec> = next.leaves[pos + 1].recs.drain(..n).collect();
        next.leaves[pos].recs.extend(moved);
        next.leaves[pos].slot = None;
        next.leaves[pos + 1].slot = None;
        next.leaves[pos + 1].sep = next.leaves[pos + 1].recs[0].ino;
        self.stats.add("borrows", 1);
        let lid = next.leaves[pos + 1].id;
        self.merge_until(next, lid);
    }

    /// unlink 最后一批：删那一条；删空就摘（写退役）；M3 再照 E176 并 / 借。M1（`crates/` 今天）只删空即摘。
    fn remove_record(&mut self, next: &mut State, ino: u64) {
        let p = next.leaf_of(ino).expect("要删的记录在树里");
        let k = next.leaves[p].recs.binary_search_by_key(&ino, |r| r.ino).unwrap();
        next.leaves[p].recs.remove(k);
        next.leaves[p].slot = None;
        if next.leaves[p].recs.is_empty() {
            let gone = next.leaves.remove(p);
            self.retire(next, &gone);
            self.stats.add("drops", 1);
            if self.m3 && p > 0 && p < next.leaves.len() { let id = next.leaves[p - 1].id; self.merge_until(next, id); }
        } else if self.m3 {
            let id = next.leaves[p].id;
            if !self.merge_until(next, id) { self.borrow(next, id); }
        }
    }

    fn leaf_recs(&self, slot: usize) -> &[Rec] {
        match &self.disk[slot].as_deref().unwrap().payload { Payload::Leaf { recs, .. } => recs, Payload::Ret { .. } => unreachable!() }
    }

    /// 扫盘重建最新那一版，按六种退役读法与真值比（K2、K3）；再对这一版判 K5 的候选判据（K5）。
    /// 叶：已发布谓词 P2（单元写序实例 + 实例表行）∧ 诞生代号在谱系父链上，同身份取 (诞生代号, 实例, 槽) 最大。
    /// 退役容器：只按 P2 择版本（随现行的树，用户定的 X1 甲）。退役读法：
    /// both = 每条在谱系上的记录都生效（追加、同身份两条都留）；first = 同身份只认最先那条（写者「已有就不追加」）；
    /// over = 同身份只认最后那条（写者「按身份覆盖」）；lastw = 死亡 txg 取被退役那片最后一版的诞生代号；
    /// nofilter = 不按谱系滤（对照，第二轮已知误判死亡）；none = 不读退役（对照，已知复活）。
    pub fn check(&mut self) {
        let chain = self.cur.chain();
        let at = format!("seq={} txg={} inst={}", self.cur.seq, self.cur.txg, self.cur.inst);
        let truth: HashMap<u64, (Txg, u32)> = self.cur.leaves.iter().flat_map(|l| l.recs.iter()).map(|r| (r.ino, (r.obirth, r.nlink))).collect();
        type Groups = HashMap<Ident, (Txg, Inst, usize)>;
        fn newest(g: &mut Groups, id: Ident, birth: Txg, inst: Inst, slot: usize) {
            let e = g.entry(id).or_insert((birth, inst, slot));
            if (birth, inst, slot) > *e { *e = (birth, inst, slot); }
        }
        let (mut leaf_g, mut ret_g): (Groups, Groups) = Default::default();
        for (slot, u) in self.disk.iter().enumerate() {
            let Some(u) = u.as_deref() else { continue };
            if !self.published(u) { continue; }
            match &u.payload {
                Payload::Leaf { id, .. } => if chain.contains(&u.birth) { newest(&mut leaf_g, *id, u.birth, u.inst, slot) },
                Payload::Ret { id, .. } => newest(&mut ret_g, *id, u.birth, u.inst, slot),
            }
        }
        let mut chosen: Vec<(Ident, usize)> = ret_g.iter().map(|(id, (_, _, s))| (*id, *s)).collect();
        chosen.sort();
        let recs: Vec<RetRec> = chosen.iter().flat_map(|(_, s)| match &self.disk[*s].as_deref().unwrap().payload {
            Payload::Ret { recs, .. } => recs.clone(), Payload::Leaf { .. } => unreachable!() }).collect();
        let mut judged = Vec::new();
        for reader in ["both", "first", "over", "lastw", "nofilter", "none"] {
            let mut retired: HashSet<Ident> = HashSet::new();
            match reader {
                "both" => retired.extend(recs.iter().filter(|r| chain.contains(&r.death)).map(|r| r.who)),
                "first" | "over" => {
                    let mut pick: HashMap<Ident, RetRec> = HashMap::new();
                    for r in &recs { if reader == "over" || !pick.contains_key(&r.who) { pick.insert(r.who, *r); } }
                    retired.extend(pick.values().filter(|r| chain.contains(&r.death)).map(|r| r.who));
                }
                "lastw" => retired.extend(recs.iter().filter(|r| chain.contains(&r.last_write)).map(|r| r.who)),
                "nofilter" => retired.extend(recs.iter().map(|r| r.who)),
                _ => {}
            }
            let mut rebuilt: HashMap<u64, (Rec, (Txg, Inst, usize))> = HashMap::new();
            for (id, key) in &leaf_g {
                if retired.contains(id) { continue; }
                for rec in self.leaf_recs(key.2) {
                    match rebuilt.get(&rec.ino) { Some((_, k)) if *k >= *key => {} _ => { rebuilt.insert(rec.ino, (*rec, *key)); } }
                }
            }
            let r = rebuilt.iter().filter(|(ino, (rec, _))| truth.get(ino) != Some(&(rec.obirth, rec.nlink))).count() as u64;
            let m = truth.keys().filter(|ino| !rebuilt.contains_key(ino)).count() as u64;
            judged.push((reader, r, m));
        }
        for (reader, r, m) in judged { self.stats.judge(reader, "inode", r, m, &at); }
        self.k5(&chain, &at);
    }

    /// K5 的候选判据，逐条对这一版（合法镜像）判：红一次就是在合法镜像上误报。
    fn k5(&mut self, chain: &HashSet<Txg>, at: &str) {
        let live: HashSet<Ident> = self.cur.leaves.iter().map(|l| l.id).collect();
        let recs = self.cur.ret_records();
        fn dup<'a>(rs: impl Iterator<Item = &'a RetRec>) -> bool { let mut seen = HashSet::new(); for r in rs { if !seen.insert(r.who) { return true; } } false }
        let mut cs: Vec<&RetC> = self.cur.rets.iter().collect();
        cs.sort_by_key(|c| c.id.num);
        let cap = self.cfg.ret_cap;
        let verdicts = [
            ("k5a_retired_identity_not_a_live_leaf_all_records", recs.iter().any(|r| live.contains(&r.who))),
            ("k5b_retired_identity_not_a_live_leaf_on_chain", recs.iter().any(|r| chain.contains(&r.death) && live.contains(&r.who))),
            ("k5c_death_txg_at_most_this_version", recs.iter().any(|r| r.death > self.cur.txg)),
            ("k5d_death_txg_at_least_container_birth", recs.iter().any(|r| r.death < r.who.birth)),
            ("k5d_strict_death_txg_after_container_birth", recs.iter().any(|r| r.death <= r.who.birth)),
            ("k5e_identity_unique_all_records", dup(recs.iter())),
            ("k5f_identity_unique_on_chain", dup(recs.iter().filter(|r| chain.contains(&r.death)))),
            ("k5g_non_rightmost_container_full", cs.len() > 1 && cs[..cs.len() - 1].iter().any(|c| c.recs.len() != cap)),
            ("k5h_container_numbers_one_to_n", cs.iter().enumerate().any(|(i, c)| c.id.num != i as u64 + 1)),
        ];
        for (name, red) in verdicts { self.stats.judge(name, "k5", u64::from(red), 0, at); }
    }
}
