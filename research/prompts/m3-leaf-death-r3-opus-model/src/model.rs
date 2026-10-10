//! m3-leaf-death-r3 攻方腿的模型（由第二轮攻方模型 `research/prompts/m3-leaf-death-r2-opus-model/src/model.rs` 改出）：
//! inode 叶容器（类型 2）、数据单元、退役记录容器（类型 3）、墓碑容器（类型 1）落在槽里；发布、崩溃（在飞发布的单元写逐子集枚举）、
//! 丢根、管理员回退、unlink 分批、合并与借、截短（不重写留下的单元）都在这里。第三轮加的：
//! - 三个候选的写路径与读法：候选一甲（退役树随现行、退役容器只按 P2 择、记录按死亡 txg 滤）、候选一乙（退役树回退取 R_old、统一按谱系滤）、
//!   候选二（删空 / 被吸收的叶留零条目版本、不退役）、TSZ（意图那次的整对象墓碑在所有版本里不收 + 同号取诞生代号最大 + 数据单元门 Z）；
//! - 副本 w = 2：每个单元两份写在两块盘上，两块盘的分配器各自取槽（垃圾在两块盘上活得不一样久）；单份坏（潜伏扇区错，过几次发布修回）；
//! - 实例表清扫：一个实例的根全部轮出根环、这一轮没有坏副本时，抹掉按它那一行判未发布的单元，再删掉那一行（之后它的单元一律当已发布）。
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
pub struct Leaf { pub id: Ident, pub recs: Vec<Rec>, pub slot: Option<usize> }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetRec { pub who: Ident, pub death: Txg }

#[derive(Clone, Debug)]
pub struct RetC { pub id: Ident, pub recs: Vec<RetRec>, pub slot: Option<usize> }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TombRec { pub ino: u64, pub obirth: Txg, pub death: Txg }

#[derive(Clone, Debug)]
pub struct TombC { pub id: Ident, pub recs: Vec<TombRec>, pub slot: Option<usize> }

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
    pub tombs: Vec<TombC>,
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
        for t in &self.tombs { s.insert(t.slot.expect("已发布的墓碑容器有槽")); }
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
    Tomb { id: Ident, recs: Vec<TombRec> },
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

/// 写路径：候选一 / TSZ 共用一条（退役记录与意图那次的墓碑都写，读法各取所需），差在管理员回退那次退役树取谁；候选二另一条（留零条目版本）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WritePath { RetCarried, RetROld, Stubs }

#[derive(Clone, Debug)]
pub struct Config {
    pub name: String,
    pub seed: u64,
    pub geometry: Geometry,
    pub ring: u64,
    pub leaf_cap: usize,
    pub ret_cap: usize,
    pub tomb_cap: usize,
    pub borrow: BorrowArm,
    pub path: WritePath,
    pub initial_inodes: u64,
    pub create_max: u64,
    pub actions: u32,
    pub p_create: u64,
    pub p_write: u64,
    pub p_append: u64,
    pub p_rollback: u64,
    pub p_crash: u64,
    pub p_lost_root: u64,
    pub crash_k_max: u32,
    /// K2 的历史：写类动作里截短（不重写留下的单元，含截到 0）的千分比；截短 / 长回来优先挑最近被挪过的记录、盯着的那片叶里的对象。
    pub p_truncate: u64,
    pub p_target_moved: u64,
    pub p_target_focus: u64,
    /// 副本 w = 2：每次发布之后掷单份坏的千分比（0 = 只有一块盘）。
    pub w2: bool,
    pub p_damage: u64,
    pub repair_after: u64,
    /// 实例表清扫。
    pub sweep: bool,
    /// 阳性对照：清扫只删行、不抹按那一行判未发布的单元（清扫写错的那一形）。
    pub sweep_without_erasing: bool,
}

#[derive(Clone, Default, Debug)]
pub struct Cell { pub scans: u64, pub r_peak: u64, pub m_peak: u64, pub q_peak: u64, pub r_scans: u64, pub m_scans: u64, pub first: Option<String> }

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
            e.r_peak = e.r_peak.max(c.r_peak); e.m_peak = e.m_peak.max(c.m_peak); e.q_peak = e.q_peak.max(c.q_peak);
            if e.first.is_none() { e.first.clone_from(&c.first); }
        }
        for (k, v) in &other.counts {
            if k.ends_with("_peak") { self.peak(k, *v); } else { self.add(k, *v); }
        }
    }
    fn judge(&mut self, reader: &str, tree: &str, r: u64, m: u64, q: u64, at: &str) {
        let e = self.cells.entry((reader.to_string(), tree.to_string())).or_default();
        e.scans += 1;
        if r > 0 { e.r_scans += 1; }
        if m > 0 { e.m_scans += 1; }
        e.r_peak = e.r_peak.max(r);
        e.m_peak = e.m_peak.max(m);
        e.q_peak = e.q_peak.max(q);
        if (r > 0 || m > 0) && e.first.is_none() { e.first = Some(format!("{at} r={r} m={m}")); }
    }
}

#[derive(Clone)]
pub struct World {
    pub cfg: Config,
    /// 两块盘；只有一块盘（w2 关）时第二块一直是空的。
    pub disk: [Vec<Option<Arc<Unit>>>; 2],
    pub alloc: Alloc,
    pub alloc1: Alloc,
    /// 盘 0 的槽 → 同一次写在盘 1 上的槽；盘 1 的槽 → 盘 0 的槽。
    pub m01: Vec<usize>,
    pub m10: Vec<usize>,
    /// 单份坏：(盘, 槽, 坏在第几次发布, 被读不出的那一份)。修回或被盖写时移走。
    pub damaged: Vec<(usize, usize, u64, Arc<Unit>)>,
    pub cur: State,
    pub line: Vec<State>,
    /// 根环：(seq, txg, 读得出, 实例)。
    pub ring: VecDeque<(u64, Txg, bool, Inst)>,
    pub rows: BTreeMap<Inst, Txg>,
    pub seq: u64,
    pub next_txg: Txg,
    pub inst_now: Inst,
    pub max_inst: Inst,
    pub rng: Rng,
    pub rng_w2: Rng,
    pub content_counter: u64,
    pub stats: Stats,
    pub inject: bool,
    pub batch_k: usize,
    pub recent_moved: Vec<u64>,
    pub recent_truncated: Vec<u64>,
}

pub enum Recovery { Crash, Lost { back: usize, jump: bool } }

fn upd<K: std::hash::Hash + Eq>(m: &mut HashMap<K, (Txg, Inst, usize)>, k: K, v: (Txg, Inst, usize)) {
    let e = m.entry(k).or_insert(v);
    if v > *e { *e = v; }
}

impl World {
    pub fn new(cfg: Config) -> Self {
        let rng = Rng(cfg.seed ^ 0x5EED_0000_0000_0001);
        let rng_w2 = Rng(cfg.seed ^ 0x0000_00D2_0000_0002);
        let (geometry, ring) = (cfg.geometry, cfg.ring);
        let g1 = if geometry == Geometry::Wide { Geometry::Wide } else { Geometry::RingRandom };
        let empty = State { txg: 0, inst: 1, seq: 0, leaves: Vec::new(), data: BTreeMap::new(), rets: Vec::new(), tombs: Vec::new(), queue: None, watermark: 1, lineage: Vec::new() };
        let mut w = World {
            cfg, disk: [Vec::new(), Vec::new()], alloc: Alloc::new(geometry, ring), alloc1: Alloc::new(g1, ring), m01: Vec::new(), m10: Vec::new(), damaged: Vec::new(),
            cur: empty.clone(), line: vec![empty], ring: VecDeque::new(), rows: BTreeMap::new(), seq: 1, next_txg: 1, inst_now: 1, max_inst: 1,
            rng, rng_w2, content_counter: 0, stats: Stats::default(), inject: true, batch_k: 1, recent_moved: Vec::new(), recent_truncated: Vec::new(),
        };
        w.ring.push_back((0, 0, true, 1));
        w
    }

    fn dirty_count(next: &State) -> usize {
        next.leaves.iter().filter(|l| l.slot.is_none()).count()
            + next.data.values().flatten().filter(|a| a.slot.is_none()).count()
            + next.rets.iter().filter(|r| r.slot.is_none()).count()
            + next.tombs.iter().filter(|r| r.slot.is_none()).count()
    }

    fn put(&mut self, d: usize, slot: usize, u: Arc<Unit>) {
        if self.disk[d].len() <= slot { self.disk[d].resize_with(slot + 1, || None); }
        self.disk[d][slot] = Some(u);
        self.damaged.retain(|e| !(e.0 == d && e.1 == slot));
    }

    /// 把 `next` 里没有槽的单元写出去；`mask` 给了就只写第 i 位为 1 的那几刀（w2 时每个单元两刀：盘 0 一刀、盘 1 一刀）。
    fn write_dirty(&mut self, next: &mut State, mask: Option<u64>) {
        let (txg, inst, seq) = (next.txg, self.inst_now, self.seq);
        let w2 = self.cfg.w2;
        let births: HashMap<u64, Txg> = next.leaves.iter().flat_map(|l| l.recs.iter().map(|r| (r.ino, r.obirth))).collect();
        let mut payloads: Vec<(Payload, &'static str)> = Vec::new();
        let mut slots: Vec<&mut Option<usize>> = Vec::new();
        for leaf in &mut next.leaves { if leaf.slot.is_none() { payloads.push((Payload::Leaf { id: leaf.id, recs: leaf.recs.clone() }, "units_leaf")); slots.push(&mut leaf.slot); } }
        for (ino, anchors) in &mut next.data {
            for (k, a) in anchors.iter_mut().enumerate() {
                if a.slot.is_none() { payloads.push((Payload::Data { ino: *ino, obirth: births[ino], anchor: k as u32, content: a.content }, "units_data")); slots.push(&mut a.slot); }
            }
        }
        for r in &mut next.rets { if r.slot.is_none() { payloads.push((Payload::Ret { id: r.id, recs: r.recs.clone() }, "units_retirement")); slots.push(&mut r.slot); } }
        for t in &mut next.tombs { if t.slot.is_none() { payloads.push((Payload::Tomb { id: t.id, recs: t.recs.clone() }, "units_tombstone")); slots.push(&mut t.slot); } }
        let copies = if w2 { 2 } else { 1 };
        let mut writes: Vec<(usize, usize, usize, Arc<Unit>)> = Vec::new();
        for (i, ((payload, what), slot)) in payloads.into_iter().zip(slots).enumerate() {
            let s0 = self.alloc.alloc(seq, &mut self.rng);
            let s1 = if w2 { self.alloc1.alloc(seq, &mut self.rng_w2) } else { 0 };
            *slot = Some(s0);
            let u = Arc::new(Unit { birth: txg, inst, payload });
            for c in 0..copies {
                let bit = i * copies + c;
                if mask.map_or(true, |m| bit >= 64 || m >> bit & 1 == 1) { writes.push((c, if c == 0 { s0 } else { s1 }, if c == 0 { s1 } else { s0 }, Arc::clone(&u))); }
            }
            if mask.is_none() { self.stats.add(what, 1); }
        }
        for (d, s, other, u) in writes {
            self.put(d, s, u);
            let m = if d == 0 { &mut self.m01 } else { &mut self.m10 };
            if m.len() <= s { m.resize(s + 1, usize::MAX); }
            m[s] = other;
        }
    }

    fn referenced1(&self, refs0: &HashSet<usize>) -> HashSet<usize> {
        refs0.iter().filter_map(|s| self.m01.get(*s).copied()).filter(|s| *s != usize::MAX).collect()
    }

    fn adopt_all(&mut self, s: &State) {
        let refs0 = s.referenced_slots();
        let refs1 = self.referenced1(&refs0);
        self.alloc.adopt(&refs0, self.seq);
        if self.cfg.w2 { self.alloc1.adopt(&refs1, self.seq); }
    }

    /// 一次发布：注入打开时先掷崩溃（在飞发布的写逐子集枚举，每个子集各开一支：恢复、挂载、续做，每次发布之后都扫）；
    /// 不崩就整写、记根，再掷单份坏、修回、清扫、扫描、丢根。交回这次发布做没做成。
    pub fn publish(&mut self, mut next: State, what: &str) -> bool {
        next.txg = self.next_txg;
        next.inst = self.inst_now;
        next.seq = self.seq;
        let k = Self::dirty_count(&next) as u32 * if self.cfg.w2 { 2 } else { 1 };
        if k >= 1 && self.inject && self.rng.chance(self.cfg.p_crash) {
            if k > self.cfg.crash_k_max {
                self.stats.add("crash_points_skipped_more_writes_than_k_max", 1);
            } else {
                self.stats.add("crash_points", 1);
                self.stats.add("crash_states_enumerated", 1u64 << k);
                for mask in 0..(1u64 << k) {
                    let mut branch = self.clone();
                    branch.inject = false;
                    branch.stats = Stats::default();
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
        self.write_dirty(&mut next, None);
        self.adopt_all(&next);
        self.ring.push_back((self.seq, next.txg, true, self.inst_now));
        while self.ring.len() as u64 > self.cfg.ring { self.ring.pop_front(); }
        self.seq += 1;
        self.next_txg += 1;
        self.cur = next.clone();
        self.line.push(next);
        let keep = self.cfg.ring as usize + 8;
        if self.line.len() > keep { let cut = self.line.len() - keep; self.line.drain(..cut); }
        self.stats.add("publishes", 1);
        if self.cfg.w2 { self.damage_and_repair(); }
        if self.cfg.sweep { self.sweep(); }
        self.check(what);
        if self.inject && self.line.len() > 4 && self.rng.chance(self.cfg.p_lost_root) {
            let variant = self.rng.below(3);
            let (back, jump) = match variant { 0 => (1, false), 1 => (1, true), _ => (3, false) };
            self.stats.add(match variant { 0 => "lost_root_one", 1 => "lost_root_one_readable_but_unusable", _ => "lost_root_three" }, 1);
            self.recover(Recovery::Lost { back, jump });
            return false;
        }
        true
    }

    /// 读得出的那一份：盘 0 那一份在就取它，否则取盘 1 上同一次写的那一份。
    fn readable_at(&self, s0: usize) -> Option<&Arc<Unit>> {
        if let Some(Some(u)) = self.disk[0].get(s0) { return Some(u); }
        let s1 = *self.m01.get(s0)?;
        let u = self.disk[1].get(s1)?.as_ref()?;
        if self.m10.get(s1) == Some(&s0) { Some(u) } else { None }
    }

    fn other_copy_readable(&self, d: usize, s: usize, u: &Arc<Unit>) -> bool {
        let (od, os) = if d == 0 { (1, self.m01.get(s).copied()) } else { (0, self.m10.get(s).copied()) };
        os.and_then(|os| self.disk[od].get(os)).and_then(|x| x.as_ref()).is_some_and(|x| Arc::ptr_eq(x, u))
    }

    /// 根环窗口里任一版（含现行）引用的单元：单份坏不许让它两份都读不出（w = 2 的承诺）；别的单元（垃圾）两份都可以坏。
    fn protected(&self, u: &Arc<Unit>) -> bool {
        self.line.iter().chain(std::iter::once(&self.cur)).any(|st| st.referenced_slots().iter().any(|&s| self.readable_at(s).is_some_and(|x| Arc::ptr_eq(x, u))))
    }

    fn damage_and_repair(&mut self) {
        let now = self.seq;
        // 修回：坏了 repair_after 次发布的那一份，另一份还读得出就照它写回原槽；读不出就那一份永远丢了。
        let due: Vec<(usize, usize, u64, Arc<Unit>)> = self.damaged.iter().filter(|e| e.2 + self.cfg.repair_after <= now).cloned().collect();
        for (d, s, _, u) in due {
            self.damaged.retain(|e| !(e.0 == d && e.1 == s));
            if self.other_copy_readable(d, s, &u) { self.disk[d][s] = Some(u); self.stats.add("w2_copies_repaired", 1); } else { self.stats.add("w2_copies_lost_for_good", 1); }
        }
        if !self.rng_w2.chance(self.cfg.p_damage) { return; }
        let d = self.rng_w2.below(2) as usize;
        let n = self.disk[d].len() as u64;
        if n == 0 { return; }
        let s = self.rng_w2.below(n) as usize;
        let Some(u) = self.disk[d][s].clone() else { return };
        if !self.other_copy_readable(d, s, &u) && self.protected(&u) { self.stats.add("w2_damage_refused_last_copy_of_a_ring_unit", 1); return; }
        self.disk[d][s] = None;
        self.damaged.push((d, s, now, u));
        self.stats.add("w2_damage_events", 1);
    }

    /// 实例表清扫：这一轮没有坏副本（crates：有一次读失败这一轮一行都不删）；一个实例不是现行实例、根环里没有它的根，
    /// 就抹掉两块盘上按它那一行判未发布的单元，再删掉那一行。
    fn sweep(&mut self) {
        if !self.damaged.is_empty() { self.stats.add("sweep_rounds_skipped_for_a_bad_copy", 1); return; }
        let rows: Vec<(Inst, Txg)> = self.rows.iter().map(|(a, b)| (*a, *b)).collect();
        for (i, t) in rows {
            if i == self.inst_now || self.ring.iter().any(|e| e.3 == i) { continue; }
            let mut erased = 0;
            if self.cfg.sweep_without_erasing { self.rows.remove(&i); self.stats.add("sweep_rows_deleted_without_erasing", 1); continue; }
            for d in 0..2 {
                for slot in self.disk[d].iter_mut() {
                    if slot.as_ref().is_some_and(|u| u.inst == i && u.birth > t) { *slot = None; erased += 1; }
                }
            }
            self.rows.remove(&i);
            self.stats.add("sweep_rows_deleted", 1);
            self.stats.add("sweep_units_erased", erased);
        }
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
        self.adopt_all(&sel);
        self.verify_intact(&sel);
        self.cur = sel;
        self.stats.add("recoveries", 1);
        let s = self.cur.clone();
        if !self.publish(s, "mount instance row") { return; }
        let s = self.cur.clone();
        if !self.publish(s, "mount warm-up") { return; }
        self.continue_queue();
    }

    fn verify_intact(&self, s: &State) {
        let leaf_ok = |l: &Leaf| matches!(self.readable_at(l.slot.unwrap()).map(|u| &u.payload), Some(Payload::Leaf { id, recs }) if *id == l.id && *recs == l.recs);
        for l in &s.leaves { assert!(leaf_ok(l), "模型错：回到的那一版的叶 {:?} 读不出", l.id); }
        for (ino, anchors) in &s.data {
            for (k, a) in anchors.iter().enumerate() {
                let ok = matches!(self.readable_at(a.slot.unwrap()).map(|u| &u.payload), Some(Payload::Data { ino: i, anchor, content, .. }) if i == ino && *anchor == k as u32 && *content == a.content);
                assert!(ok, "模型错：回到的那一版的数据单元 {ino}/{k} 读不出");
            }
        }
    }

    pub fn rollback(&mut self) -> bool {
        let n = self.line.len();
        let candidates: Vec<usize> = (0..n.saturating_sub(1)).filter(|&i| self.seq - self.line[i].seq + 2 <= self.cfg.ring && self.line[i].txg > 0).collect();
        if candidates.is_empty() { return true; }
        let i = candidates[self.rng.below(candidates.len() as u64) as usize];
        self.rollback_to(i)
    }

    pub fn rollback_to(&mut self, i: usize) -> bool {
        let target = self.line[i].clone();
        self.stats.add("rollbacks", 1);
        let mut next = target.clone();
        if self.cfg.path != WritePath::RetROld { next.rets = self.cur.rets.clone(); }
        next.tombs = self.cur.tombs.clone();
        next.watermark = self.cur.watermark.max(target.watermark);
        next.lineage = self.cur.lineage.clone();
        next.lineage.push((self.next_txg, target.txg));
        self.verify_intact(&target);
        self.line.truncate(i + 1);
        if !self.publish(next, "admin rollback") { return false; }
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
                _ => next.leaves.push(Leaf { id: Ident { num: ino, birth: txg }, recs: vec![rec], slot: None }),
            }
        }
        self.publish(next, "create")
    }

    fn set_record(next: &mut State, rec: Rec) {
        let p = next.leaf_of(rec.ino).expect("记录在树里");
        let l = &mut next.leaves[p];
        let k = l.recs.binary_search_by_key(&rec.ino, |r| r.ino).unwrap();
        l.recs[k] = rec;
        l.slot = None;
    }

    /// 覆盖写：写 u 个单元；`same_size` 时 size 与单元数不变、每个单元换一份新内容。
    pub fn overwrite(&mut self, ino: u64, units: u64, same_size: bool) -> bool {
        let mut next = self.cur.clone();
        let mut rec = next.record(ino).unwrap();
        let (u, size) = if same_size && rec.size > 0 { (units_of(rec.size), rec.size) } else { (units, units * PAYLOAD - self.rng.below(PAYLOAD)) };
        let anchors: Vec<Anchor> = (0..u).map(|_| Anchor { content: self.new_content(), slot: None }).collect();
        next.data.insert(ino, anchors);
        rec.size = size;
        Self::set_record(&mut next, rec);
        self.publish(next, "overwrite")
    }

    /// 截短到 u 个单元（u 可以是 0）：留下的单元不重写，尾部那几个单元离开这一版；size 落在 u 个单元里。
    pub fn truncate(&mut self, ino: u64, u: u64) -> bool {
        let mut next = self.cur.clone();
        let mut rec = next.record(ino).unwrap();
        let anchors = next.data.entry(ino).or_default();
        anchors.truncate(u as usize);
        if anchors.is_empty() { next.data.remove(&ino); }
        rec.size = if u == 0 { 0 } else { u * PAYLOAD - self.rng.below(PAYLOAD) };
        Self::set_record(&mut next, rec);
        self.stats.add(if u == 0 { "truncates_to_zero" } else { "truncates" }, 1);
        if self.recent_moved.contains(&ino) { self.stats.add("truncates_of_a_record_moved_to_another_leaf_before", 1); }
        self.recent_truncated.push(ino);
        if self.recent_truncated.len() > 32 { self.recent_truncated.remove(0); }
        self.publish(next, "truncate")
    }

    /// 顺序写：在末尾追加 more 个单元（文件没有洞：单元数 = ⌈size ÷ 净荷⌉，最后一个单元原样不重写）。
    pub fn append(&mut self, ino: u64, more: u64) -> bool {
        let mut next = self.cur.clone();
        let mut rec = next.record(ino).unwrap();
        let u0 = units_of(rec.size);
        if u0 == 0 && self.recent_truncated.contains(&ino) { self.stats.add("regrows_after_a_truncate", 1); }
        let anchors = next.data.entry(ino).or_default();
        assert_eq!(anchors.len() as u64, u0, "文件没有洞：单元数 = ⌈size ÷ 净荷⌉");
        for _ in 0..more { self.content_counter += 1; anchors.push(Anchor { content: self.content_counter, slot: None }); }
        rec.size = (u0 + more) * PAYLOAD;
        Self::set_record(&mut next, rec);
        self.publish(next, "append")
    }

    fn tombstone(&mut self, next: &mut State, ino: u64, obirth: Txg) {
        let death = self.next_txg;
        let cap = self.cfg.tomb_cap;
        match next.tombs.last_mut() {
            Some(open) if open.recs.len() < cap => { open.recs.push(TombRec { ino, obirth, death }); open.slot = None; }
            _ => {
                let num = next.tombs.iter().map(|r| r.id.num).max().unwrap_or(0) + 1;
                next.tombs.push(TombC { id: Ident { num, birth: death }, recs: vec![TombRec { ino, obirth, death }], slot: None });
            }
        }
    }

    pub fn unlink(&mut self, ino: u64) -> bool {
        let mut next = self.cur.clone();
        let mut rec = next.record(ino).unwrap();
        rec.nlink = 0;
        Self::set_record(&mut next, rec);
        next.queue = Some(ino);
        self.tombstone(&mut next, ino, rec.obirth);
        self.batch_k = 1 + self.rng.below(3) as usize;
        if !self.publish(next, "unlink intent") { return false; }
        self.continue_queue()
    }

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
                if !self.publish(next, "unlink batch") { return false; }
            } else {
                next.data.remove(&ino);
                next.queue = None;
                self.remove_record(&mut next, ino);
                if !self.publish(next, "unlink last batch") { return false; }
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

    fn moved(&mut self, next: &State, recs: &[Rec]) {
        let n = recs.iter().filter(|r| next.data.get(&r.ino).is_some_and(|a| !a.is_empty())).count() as u64;
        self.stats.add("records_with_data_moved_to_another_leaf", n);
        self.recent_moved.extend(recs.iter().map(|r| r.ino));
        let over = self.recent_moved.len().saturating_sub(64);
        self.recent_moved.drain(..over);
    }

    /// unlink 最后一批：删那一条，再照 M2 / M3 均分借的写路径处理那一片。候选二（`Stubs`）：删空的那片留零条目版本，被吸收的右半也留零条目版本，
    /// 合并、借只在有记录的片之间做（零条目桩跳过），不写退役记录。
    fn remove_record(&mut self, next: &mut State, ino: u64) {
        let cap = self.cfg.leaf_cap;
        let floor = cap / 2;
        let stubs = self.cfg.path == WritePath::Stubs;
        let p = next.leaf_of(ino).expect("要删的记录在树里");
        let k = next.leaves[p].recs.binary_search_by_key(&ino, |r| r.ino).unwrap();
        next.leaves[p].recs.remove(k);
        next.leaves[p].slot = None;
        let len = |s: &State, i: usize| s.leaves[i].recs.len();
        if next.leaves[p].recs.is_empty() {
            self.stats.add("drops", 1);
            if stubs { self.stats.add("zero_entry_versions", 1); return; }
            let gone = next.leaves.remove(p);
            self.retire(next, gone.id);
            for l in next.leaves.iter_mut().skip(p) { l.slot = None; }
            return;
        }
        let right = (p + 1..next.leaves.len()).find(|&j| !next.leaves[j].recs.is_empty());
        let left = (0..p).rev().find(|&j| !next.leaves[j].recs.is_empty());
        if let Some(j) = right.filter(|&j| len(next, p) + len(next, j) <= cap) {
            let recs = std::mem::take(&mut next.leaves[j].recs);
            self.moved(next, &recs);
            next.leaves[p].recs.extend(recs);
            self.stats.add("absorbs", 1);
            if stubs { next.leaves[j].slot = None; self.stats.add("zero_entry_versions", 1); return; }
            let gone = next.leaves.remove(j);
            self.retire(next, gone.id);
            for l in next.leaves.iter_mut().skip(p) { l.slot = None; }
            return;
        }
        if let Some(j) = left.filter(|&j| len(next, j) + len(next, p) <= cap) {
            let recs = std::mem::take(&mut next.leaves[p].recs);
            self.moved(next, &recs);
            next.leaves[j].recs.extend(recs);
            next.leaves[j].slot = None;
            self.stats.add("absorbs", 1);
            if stubs { self.stats.add("zero_entry_versions", 1); return; }
            let gone = next.leaves.remove(p);
            self.retire(next, gone.id);
            for l in next.leaves.iter_mut().skip(j) { l.slot = None; }
            return;
        }
        if self.cfg.borrow != BorrowArm::NoBorrow && len(next, p) < floor {
            if let Some(j) = right {
                let n = (len(next, j) - len(next, p)) / 2;
                if n >= 1 {
                    let moved: Vec<Rec> = next.leaves[j].recs.drain(..n).collect();
                    self.moved(next, &moved);
                    next.leaves[p].recs.extend(moved);
                    next.leaves[j].slot = None;
                    let new_min = next.leaves[j].recs[0].ino;
                    if self.cfg.borrow == BorrowArm::N2 {
                        assert!(!stubs, "候选二只配不借 / N1");
                        let old = next.leaves[j].id;
                        self.retire(next, old);
                        next.leaves[j].id = Ident { num: new_min, birth: self.next_txg };
                    }
                    self.stats.add("borrows", 1);
                }
            }
        }
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

    /// 扫盘重建最新那一版：读得出的单元（两块盘去重：同一次写的两份算一个单元），按读法与真值比。
    /// 每格记 r（读出来与真值不同或真值里没有的记录 / 单元数）、m（真值里有而没读出来的，不含删除队列里那个对象）、q（删除队列里那个对象没读出来的）。
    pub fn check(&mut self, what: &str) {
        let chain = self.chain();
        let at = format!("seq={} txg={} inst={} after={}", self.cur.seq, self.cur.txg, self.cur.inst, what.replace(' ', "_"));
        let mut units: Vec<&Arc<Unit>> = self.disk[0].iter().flatten().collect();
        let mut only_on_disk1 = 0u64;
        for (s1, u) in self.disk[1].iter().enumerate() {
            let Some(u) = u else { continue };
            let dup = self.m10.get(s1).and_then(|&s0| self.disk[0].get(s0)).and_then(|x| x.as_ref()).is_some_and(|x| Arc::ptr_eq(x, u));
            if !dup { units.push(u); only_on_disk1 += 1; }
        }
        let queued = self.cur.queue;
        let truth_inode: HashMap<u64, (Txg, u32, u64)> = self.cur.leaves.iter().flat_map(|l| l.recs.iter()).map(|r| (r.ino, (r.obirth, r.nlink, r.size))).collect();
        let truth_data: HashMap<(u64, Txg, u32), u64> = self.cur.data.iter().flat_map(|(ino, a)| {
            let b = truth_inode[ino].0;
            a.iter().enumerate().map(move |(k, x)| ((*ino, b, k as u32), x.content))
        }).collect();
        let mut leaf_g: HashMap<Ident, (Txg, Inst, usize)> = HashMap::new();
        let mut data_g: HashMap<(u64, Txg, u32), (Txg, Inst, usize)> = HashMap::new();
        let mut ret_p2: HashMap<Ident, (Txg, Inst, usize)> = HashMap::new();
        let mut ret_lin: HashMap<Ident, (Txg, Inst, usize)> = HashMap::new();
        let mut tomb_p2: HashMap<Ident, (Txg, Inst, usize)> = HashMap::new();
        for (idx, u) in units.iter().enumerate() {
            if !self.published(u) { continue; }
            let on = chain.contains(&u.birth);
            let key = (u.birth, u.inst, idx);
            match &u.payload {
                Payload::Ret { id, .. } => { upd(&mut ret_p2, *id, key); if on { upd(&mut ret_lin, *id, key); } }
                Payload::Tomb { id, .. } => upd(&mut tomb_p2, *id, key),
                _ if !on => {}
                Payload::Leaf { id, .. } => upd(&mut leaf_g, *id, key),
                Payload::Data { ino, obirth, anchor, .. } => upd(&mut data_g, (*ino, *obirth, *anchor), key),
            }
        }
        // (读法名, 退役：0 不读 / 1 甲（P2 择、死亡 txg 滤）/ 2 乙（谱系择、不按死亡 txg 滤）/ 3 丙（谱系择、死亡 txg 滤）, 墓碑：0 不读 / 1 TSZ / 2 TSZ 不按谱系滤)
        let readers: Vec<(&str, u8, u8)> = match self.cfg.path {
            WritePath::RetCarried => vec![("S", 0, 0), ("C1jia", 1, 0), ("C1bing_uniform", 3, 0), ("TSZ", 0, 1), ("TSZ_noLin", 0, 2)],
            WritePath::RetROld => vec![("S", 0, 0), ("C1yi", 2, 0), ("TSZ", 0, 1)],
            WritePath::Stubs => vec![("C2", 0, 0)],
        };
        let mut judged: Vec<(&str, &'static str, u64, u64, u64)> = Vec::new();
        for (name, ret, tomb) in readers {
            let mut retired: HashSet<Ident> = HashSet::new();
            if ret > 0 {
                let groups = if ret == 1 { &ret_p2 } else { &ret_lin };
                for (_, _, idx) in groups.values() {
                    if let Payload::Ret { recs, .. } = &units[*idx].payload {
                        for r in recs { if ret == 2 || chain.contains(&r.death) { retired.insert(r.who); } }
                    }
                }
            }
            let mut dead: HashSet<(u64, Txg)> = HashSet::new();
            if tomb > 0 {
                for (_, _, idx) in tomb_p2.values() {
                    if let Payload::Tomb { recs, .. } = &units[*idx].payload {
                        for t in recs { if tomb == 2 || chain.contains(&t.death) { dead.insert((t.ino, t.obirth)); } }
                    }
                }
            }
            let mut inode: HashMap<u64, (Rec, (Txg, Inst, usize))> = HashMap::new();
            for (id, key) in &leaf_g {
                if retired.contains(id) { continue; }
                let Payload::Leaf { recs, .. } = &units[key.2].payload else { unreachable!() };
                for rec in recs {
                    if dead.contains(&(rec.ino, rec.obirth)) { continue; }
                    match inode.get(&rec.ino) {
                        Some((_, k)) if *k >= *key => {}
                        _ => { inode.insert(rec.ino, (*rec, *key)); }
                    }
                }
            }
            let r = inode.iter().filter(|(ino, (rec, _))| truth_inode.get(ino) != Some(&(rec.obirth, rec.nlink, rec.size))).count() as u64;
            let m = truth_inode.keys().filter(|ino| Some(**ino) != queued && !inode.contains_key(ino)).count() as u64;
            let q = queued.filter(|q| !inode.contains_key(q)).map_or(0, |_| 1);
            judged.push((name, "inode", r, m, q));
            let kept: HashMap<(u64, Txg, u32), u64> = data_g.iter().filter(|((ino, ob, anchor), _)| {
                inode.get(ino).is_some_and(|(rec, _)| rec.obirth == *ob && u64::from(*anchor) < units_of(rec.size))
            }).map(|(k, v)| {
                let Payload::Data { content, .. } = &units[v.2].payload else { unreachable!() };
                (*k, *content)
            }).collect();
            let r = kept.iter().filter(|(k, c)| truth_data.get(k) != Some(c)).count() as u64;
            let m = truth_data.keys().filter(|k| Some(k.0) != queued && !kept.contains_key(k)).count() as u64;
            let q = truth_data.keys().filter(|k| Some(k.0) == queued && !kept.contains_key(k)).count() as u64;
            judged.push((name, "data_Z", r, m, q));
            if name == "C1jia" || name == "C2" {
                let rg: HashMap<(u64, Txg, u32), u64> = data_g.iter().filter(|((ino, ob, _), _)| inode.get(ino).is_some_and(|(rec, _)| rec.obirth == *ob)).map(|(k, v)| {
                    let Payload::Data { content, .. } = &units[v.2].payload else { unreachable!() };
                    (*k, *content)
                }).collect();
                let r = rg.iter().filter(|(k, c)| truth_data.get(k) != Some(c)).count() as u64;
                let m = truth_data.keys().filter(|k| Some(k.0) != queued && !rg.contains_key(k)).count() as u64;
                judged.push((name, "data_RG", r, m, 0));
            }
        }
        for (name, tree, r, m, q) in judged { self.stats.judge(name, tree, r, m, q, &at); }
        self.stats.peak("w2_units_readable_only_on_disk1_peak", only_on_disk1);
        let stubs = self.cur.leaves.iter().filter(|l| l.recs.is_empty()).count() as u64;
        self.stats.peak("zero_entry_leaves_peak", stubs);
        self.stats.add("scans", 1);
    }
}
