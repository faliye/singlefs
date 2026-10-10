//! m3-leaf-death-r3 正推腿（Sonnet）的模型：第二轮攻方腿模型（research/prompts/m3-leaf-death-r2-opus-model/）的拷贝加改动。
//! 改动：① 候选并排——Today（今天，读法 S）、RetA / RetB（候选一：退役记录，改法甲随现行 / 改法乙回退取 R_old）、Persist / PersistX（候选二：
//! 零条目版本一直留在叶序里，合并不跨零条目叶 / 跨）、Tsz（候选三：今天意图那次的整对象墓碑 + 供出规则 + 数据单元门）、Fsz（墓碑挪到最后一批）；
//! ② 同一段历史：用户动作与故障的随机数各自按（动作序号、本动作内第几次发布）取，与候选写出多少单元无关，逻辑状态的哈希逐段对得上；
//! ③ 判三个名字空间：inode 记录、数据单元、「单元」（分配记录 / 映射按槽号算的那一格）；漏活（m）分在 unlink 途中与途外；
//! ④ 每类写（叶、数据、退役、墓碑）按事件记账，只记主线（崩溃支线只判，不记账）。不依赖 crates/。
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

/// 退役记录与墓碑记录共用：退役时 who = 被退役的容器身份，墓碑时 who = (inode 号, 对象出生代)。
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
    pub tombs: Vec<RetC>,
    pub queue: Option<u64>,
    pub watermark: u64,
    pub lineage: Vec<(Txg, Txg)>,
}

impl State {
    pub fn referenced_slots(&self) -> HashSet<usize> {
        let mut s = HashSet::new();
        for l in &self.leaves { s.insert(l.slot.expect("已发布的叶有槽")); }
        for a in self.data.values().flatten() { s.insert(a.slot.expect("已发布的数据单元有槽")); }
        for r in self.rets.iter().chain(&self.tombs) { s.insert(r.slot.expect("已发布的退役 / 墓碑容器有槽")); }
        s
    }
    pub fn record(&self, ino: u64) -> Option<Rec> {
        self.leaves.iter().find_map(|l| l.recs.iter().find(|r| r.ino == ino).copied())
    }
    pub fn leaf_of(&self, ino: u64) -> Option<usize> {
        self.leaves.iter().position(|l| l.recs.binary_search_by_key(&ino, |r| r.ino).is_ok())
    }
    /// 逻辑状态的哈希：与叶怎么分、有没有零条目叶、退役 / 墓碑怎么存都无关。
    pub fn logical_hash(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut mix = |x: u64| { h ^= x; h = h.wrapping_mul(0x0000_0100_0000_01B3); h ^= h >> 29; };
        let mut recs: Vec<Rec> = self.leaves.iter().flat_map(|l| l.recs.iter().copied()).collect();
        recs.sort_by_key(|r| r.ino);
        for r in recs { mix(r.ino); mix(r.obirth); mix(u64::from(r.nlink)); mix(r.size); }
        for (ino, a) in &self.data { mix(*ino); for x in a { mix(x.content); } }
        mix(self.queue.unwrap_or(0));
        mix(self.watermark);
        h
    }
}

#[derive(Debug)]
pub enum Payload {
    Leaf { id: Ident, recs: Vec<Rec> },
    Data { ino: u64, obirth: Txg, anchor: u32, content: u64 },
    Ret { id: Ident, recs: Vec<RetRec> },
    Tomb { id: Ident, recs: Vec<RetRec> },
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
    /// 由 (种子, 动作序号, 本动作内第几次发布) 定下来的一条独立流。
    pub fn keyed(seed: u64, a: u64, b: u64) -> Rng {
        let mut r = Rng(seed ^ 0xA076_1D64_78BD_642F);
        r.0 ^= a.wrapping_mul(0xE703_7ED1_A0B4_28DB);
        r.next();
        r.0 ^= b.wrapping_mul(0x8EBC_6AF0_9C88_C6E3);
        r.next();
        r
    }
}

pub fn units_of(size: u64) -> u64 { size.div_ceil(PAYLOAD) }

/// 候选：Today = `crates/` 今天（删空不落盘、不写退役记录、意图那次的整对象墓碑只被读法 S 无视）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cand { Today, RetA, RetB, Persist, PersistX, Tsz, Fsz }

impl Cand {
    pub fn name(self) -> &'static str {
        match self { Cand::Today => "today", Cand::RetA => "ret_a", Cand::RetB => "ret_b", Cand::Persist => "persist", Cand::PersistX => "persist_x", Cand::Tsz => "tsz", Cand::Fsz => "fsz" }
    }
    pub fn keeps_stubs(self) -> bool { matches!(self, Cand::Persist | Cand::PersistX) }
    pub fn writes_retirements(self) -> bool { matches!(self, Cand::RetA | Cand::RetB) }
    pub fn writes_tombstones(self) -> bool { matches!(self, Cand::Today | Cand::RetA | Cand::RetB | Cand::Persist | Cand::PersistX | Cand::Tsz | Cand::Fsz) }
}

/// 借法：N1New = 分隔 key 取孩子这一版的最小 key（零条目叶退回容器号）；N2 = 借完右片换身份（容器号 = 借后最小 key、出生代 = 这次发布）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BorrowArm { NoBorrow, N1New, N2, N1Raise }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DataMode { AllInodes, OnlyInodeOne }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DelOrder { Focus, Random, Ascending, Descending }

#[derive(Clone, Debug)]
pub struct Config {
    pub name: String,
    pub seed: u64,
    pub geometry: Geometry,
    pub ring: u64,
    pub leaf_cap: usize,
    pub ret_cap: usize,
    pub tomb_cap: usize,
    pub cand: Cand,
    pub borrow: BorrowArm,
    pub data_mode: DataMode,
    pub del_order: DelOrder,
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
    /// 是否多跑两个消融读法（去掉 P2、去掉版本谱系滤）：证明装置看得见这两种错。
    pub ablate: bool,
    /// 实例表清扫：0 不清扫；1 清扫（抹掉按行判未发布的单元再删行）；2 清扫漏抹证据单元（退役 / 墓碑容器、零条目叶）就删行。
    pub sweep: u8,
}

#[derive(Clone, Default, Debug)]
pub struct Cell { pub scans: u64, pub r_peak: u64, pub m_peak: u64, pub r_scans: u64, pub m_scans: u64, pub m_scans_in_window: u64, pub m_scans_outside_window: u64, pub first: Option<String> }

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
            e.m_scans_in_window += c.m_scans_in_window; e.m_scans_outside_window += c.m_scans_outside_window;
            e.r_peak = e.r_peak.max(c.r_peak); e.m_peak = e.m_peak.max(c.m_peak);
            if e.first.is_none() { e.first.clone_from(&c.first); }
        }
        for (k, v) in &other.counts {
            if k.ends_with("_peak") { self.peak(k, *v); } else { self.add(k, *v); }
        }
    }
    fn judge(&mut self, reader: &str, tree: &str, r: u64, m: u64, at: &str, in_window: bool) {
        let e = self.cells.entry((reader.to_string(), tree.to_string())).or_default();
        e.scans += 1;
        if r > 0 { e.r_scans += 1; }
        if m > 0 { e.m_scans += 1; if in_window { e.m_scans_in_window += 1; } else { e.m_scans_outside_window += 1; } }
        e.r_peak = e.r_peak.max(r);
        e.m_peak = e.m_peak.max(m);
        if (r > 0 || m > 0) && e.first.is_none() { e.first = Some(format!("{at} r={r} m={m}{}", if in_window { " in_window" } else { "" })); }
    }
}

fn hash_pair(a: u64, b: u64) -> u64 { let mut r = Rng(a ^ 0x1234_5678_9ABC_DEF1); r.0 ^= b.wrapping_mul(0x9E37_79B9_7F4A_7C15); r.next() }

#[derive(Clone)]
pub struct World {
    pub cfg: Config,
    pub disk: Vec<Option<Arc<Unit>>>,
    pub alloc: Alloc,
    pub alloc_rng: Rng,
    pub cur: State,
    pub line: Vec<State>,
    pub ring: VecDeque<(u64, Txg, bool)>,
    pub rows: BTreeMap<Inst, Txg>,
    pub seq: u64,
    pub next_txg: Txg,
    pub inst_now: Inst,
    pub max_inst: Inst,
    pub arng: Rng,
    pub action_idx: u64,
    pub ordinal: u64,
    pub content_counter: u64,
    pub stats: Stats,
    pub inject: bool,
    pub main: bool,
    pub batch_k: usize,
    pub abandoned_objects: HashSet<(u64, Txg)>,
    pub structural_seqs: BTreeSet<u64>,
    pub history: Vec<String>,
    pub record_history: bool,
    pub force_crash: bool,
    pub dumped: bool,
    pub logical_trace: u64,
}

pub enum Recovery { Crash, Lost { back: usize, jump: bool } }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Removed { Plain, Drop, Absorb, Borrow }

impl World {
    pub fn new(cfg: Config) -> Self {
        let geometry = cfg.geometry;
        let ring = cfg.ring;
        let empty = State { txg: 0, inst: 1, seq: 0, leaves: Vec::new(), data: BTreeMap::new(), rets: Vec::new(), tombs: Vec::new(), queue: None, watermark: 1, lineage: Vec::new() };
        let seed = cfg.seed;
        let mut w = World {
            cfg, disk: Vec::new(), alloc: Alloc::new(geometry, ring), alloc_rng: Rng(seed ^ 0x5EED_0000_0000_0001), cur: empty.clone(), line: vec![empty], ring: VecDeque::new(),
            rows: BTreeMap::new(), seq: 1, next_txg: 1, inst_now: 1, max_inst: 1, arng: Rng(seed), action_idx: 0, ordinal: 0, content_counter: 0, stats: Stats::default(),
            inject: true, main: true, batch_k: 1, abandoned_objects: HashSet::new(), structural_seqs: BTreeSet::new(), history: Vec::new(), record_history: false,
            force_crash: false, dumped: false, logical_trace: 0,
        };
        w.ring.push_back((0, 0, true));
        w
    }

    /// 每个用户动作开头换一条只由（种子, 动作序号）定的流，发布内的故障流由（种子, 动作序号, 第几次发布）定：
    /// 与候选写出多少单元、读法怎么判都无关，所以各候选走的是同一段逻辑历史。
    pub fn begin_action(&mut self) {
        self.action_idx += 1;
        self.ordinal = 0;
        self.arng = Rng::keyed(self.cfg.seed, self.action_idx, 0xAAAA);
        if self.main && self.cfg.sweep != 0 { self.sweep_instance_rows(); }
        if self.main {
            let h = self.cur.logical_hash();
            self.logical_trace = hash_pair(self.logical_trace, h);
        }
    }

    /// 实例表清扫（`crates/singlefs-core/src/instance_table_sweep.rs` 的模型）：一个实例的根全部轮出之后，抹掉按它那一行判未发布的单元，再删掉那一行。
    /// 删行之后这个实例剩下的单元一律当已发布。sweep = 2 时证据单元（退役 / 墓碑容器、零条目叶）漏抹，行照删——看每个候选对「漏抹」多敏感。
    fn sweep_instance_rows(&mut self) {
        let newest_old = self.line.iter().map(|s| s.inst).min().unwrap_or(self.inst_now).min(self.inst_now);
        let swept: Vec<Inst> = self.rows.keys().copied().filter(|i| *i < newest_old).collect();
        for i in swept {
            let row = self.rows[&i];
            let mut erased = 0u64; let mut missed = 0u64;
            for slot in 0..self.disk.len() {
                let Some(u) = self.disk[slot].as_deref() else { continue };
                if u.inst != i || u.birth <= row { continue; }
                let evidence = match &u.payload { Payload::Ret { .. } | Payload::Tomb { .. } => true, Payload::Leaf { recs, .. } => recs.is_empty(), Payload::Data { .. } => false };
                if self.cfg.sweep == 2 && evidence { missed += 1; continue; }
                self.disk[slot] = None;
                erased += 1;
            }
            self.rows.remove(&i);
            self.acct("sweep_rows_deleted", 1);
            self.acct("sweep_unpublished_units_erased", erased);
            self.acct("sweep_unpublished_evidence_units_missed", missed);
        }
    }

    fn acct(&mut self, key: &str, n: u64) { if self.main { self.stats.add(key, n); } }
    fn note(&mut self, s: String) { if self.record_history { self.history.push(s); } }

    fn dirty_count(next: &State) -> usize {
        next.leaves.iter().filter(|l| l.slot.is_none()).count()
            + next.data.values().flatten().filter(|a| a.slot.is_none()).count()
            + next.rets.iter().filter(|r| r.slot.is_none()).count()
            + next.tombs.iter().filter(|r| r.slot.is_none()).count()
    }

    /// 把 `next` 里没有槽的单元写出去；`mask` 给了就只写第 i 位为 1 的那几个（崩溃时在飞发布写出的子集）。
    fn write_dirty(&mut self, next: &mut State, mask: Option<u64>, tag: &str) {
        let (txg, inst, seq) = (next.txg, self.inst_now, self.seq);
        let mut i = 0u32;
        let take = |i: &mut u32| { let t = mask.map_or(true, |m| if *i >= 64 { true } else { m >> *i & 1 == 1 }); *i += 1; t };
        let mut writes: Vec<(usize, Payload)> = Vec::new();
        let (mut leaf_units, mut data_units, mut ret_units, mut tomb_units, mut stub_units) = (0u64, 0u64, 0u64, 0u64, 0u64);
        for leaf in &mut next.leaves {
            if leaf.slot.is_none() && take(&mut i) {
                let slot = self.alloc.alloc(seq, &mut self.alloc_rng);
                leaf.slot = Some(slot);
                if leaf.recs.is_empty() { stub_units += 1; }
                writes.push((slot, Payload::Leaf { id: leaf.id, recs: leaf.recs.clone() }));
                leaf_units += 1;
            }
        }
        let births: HashMap<u64, Txg> = next.leaves.iter().flat_map(|l| l.recs.iter().map(|r| (r.ino, r.obirth))).collect();
        for (ino, anchors) in &mut next.data {
            for (k, a) in anchors.iter_mut().enumerate() {
                if a.slot.is_none() && take(&mut i) {
                    let slot = self.alloc.alloc(seq, &mut self.alloc_rng);
                    a.slot = Some(slot);
                    writes.push((slot, Payload::Data { ino: *ino, obirth: births[ino], anchor: k as u32, content: a.content }));
                    data_units += 1;
                }
            }
        }
        for r in &mut next.rets {
            if r.slot.is_none() && take(&mut i) {
                let slot = self.alloc.alloc(seq, &mut self.alloc_rng);
                r.slot = Some(slot);
                writes.push((slot, Payload::Ret { id: r.id, recs: r.recs.clone() }));
                ret_units += 1;
            }
        }
        for r in &mut next.tombs {
            if r.slot.is_none() && take(&mut i) {
                let slot = self.alloc.alloc(seq, &mut self.alloc_rng);
                r.slot = Some(slot);
                writes.push((slot, Payload::Tomb { id: r.id, recs: r.recs.clone() }));
                tomb_units += 1;
            }
        }
        for (slot, payload) in writes {
            if self.disk.len() <= slot { self.disk.resize_with(slot + 1, || None); }
            self.disk[slot] = Some(Arc::new(Unit { birth: txg, inst, payload }));
        }
        if mask.is_none() && self.main {
            self.stats.add("units_leaf", leaf_units);
            self.stats.add("units_data", data_units);
            self.stats.add("units_retirement", ret_units);
            self.stats.add("units_tombstone", tomb_units);
            self.stats.add("units_stub", stub_units);
            // 树根：退役树 / 墓碑树各自的容器写了一个就多重写一个根（照墓碑树 `plan_tombstone_append`：容器 + 根两个角色）。
            let roots = u64::from(ret_units > 0) + u64::from(tomb_units > 0);
            self.stats.add("units_tree_roots_of_the_side_trees", roots);
            self.stats.add(&format!("pub_{tag}"), 1);
            self.stats.add(&format!("leafunits_{tag}"), leaf_units);
            self.stats.add(&format!("retunits_{tag}"), ret_units);
            self.stats.add(&format!("tombunits_{tag}"), tomb_units);
            self.stats.add(&format!("dataunits_{tag}"), data_units);
        }
    }

    /// 一次发布。注入打开时先掷崩溃：在飞发布的 k 个单元写逐子集枚举（k 超过上限时取空集、全集、单点、去掉单点），
    /// 每个子集各开一支：恢复、写行、暖机、续做，每次发布之后都扫；主线取全写出那一支往下走。不崩就整写、记根，再掷丢根。
    /// 交回这次发布做没做成（崩了或丢根交回 false，调用方不再往下发）。
    pub fn publish(&mut self, mut next: State, what: &str, structural: bool) -> bool {
        next.txg = self.next_txg;
        next.inst = self.inst_now;
        next.seq = self.seq;
        let mut fr = Rng::keyed(self.cfg.seed, self.action_idx, 1 + self.ordinal);
        self.ordinal += 1;
        let want_crash = fr.chance(self.cfg.p_crash);
        let want_lost = fr.chance(self.cfg.p_lost_root);
        let variant = fr.below(3);
        let k = Self::dirty_count(&next) as u32;
        // k = 0 的发布（这个模型没建的根层单元照样要写）也能崩，只有空集一个状态：崩与不崩的掷法不随候选写多少单元而变。
        if self.force_crash || (self.inject && want_crash) {
            self.force_crash = false;
            let masks: Vec<u64> = if k <= self.cfg.crash_k_max {
                (0..(1u64 << k)).collect()
            } else if k <= 62 {
                let full = (1u64 << k) - 1;
                let mut v = vec![0, full];
                for i in 0..k { v.push(1u64 << i); v.push(full ^ (1u64 << i)); }
                v.sort_unstable(); v.dedup();
                self.acct("crash_points_sampled_not_enumerated", 1);
                v
            } else {
                self.acct("crash_points_skipped", 1);
                vec![0, u64::MAX]
            };
            self.note(format!("crash at txg {} inst {} during {what} k={k}", next.txg, self.inst_now));
            self.acct("crash_points", 1);
            if next.rets.iter().any(|r| r.slot.is_none()) { self.acct("crash_points_with_a_retirement_unit_in_flight", 1); }
            if next.tombs.iter().any(|r| r.slot.is_none()) { self.acct("crash_points_with_a_tombstone_unit_in_flight", 1); }
            if next.leaves.iter().any(|l| l.slot.is_none() && l.recs.is_empty()) { self.acct("crash_points_with_a_zero_entry_leaf_in_flight", 1); }
            self.acct("crash_states_enumerated", masks.len() as u64);
            if self.main { self.stats.peak("crash_k_peak", u64::from(k)); }
            let created: Vec<(u64, Txg)> = Self::created_objects(&self.cur, &next);
            for mask in masks {
                let mut branch = self.clone();
                branch.inject = false;
                branch.main = false;
                branch.stats = Stats::default();
                branch.record_history = false;
                let mut n = next.clone();
                branch.write_dirty(&mut n, Some(mask), what);
                branch.seq += 1;
                branch.recover(Recovery::Crash);
                let s = branch.stats.clone();
                self.stats.merge(&s);
            }
            let mut n = next.clone();
            self.write_dirty(&mut n, Some(u64::MAX), what);
            self.seq += 1;
            for o in created { self.abandoned_objects.insert(o); }
            self.recover(Recovery::Crash);
            return false;
        }
        let created: Vec<(u64, Txg)> = Self::created_objects(&self.cur, &next);
        for o in &created {
            if self.abandoned_objects.contains(o) { self.acct("objects_reissued_with_an_abandoned_number_and_birth", 1); }
        }
        if self.cfg.borrow == BorrowArm::N1New {
            // N1 新形：分隔 key = 孩子这一版的最小 key，孩子每重写一次父条目跟着重算（零条目叶没有最小 key，保留原值）。
            for l in &mut next.leaves { if l.slot.is_none() { if let Some(first) = l.recs.first() { l.sep = first.ino; } } }
        }
        self.write_dirty(&mut next, None, what);
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
        self.acct("publishes", 1);
        self.check();
        if self.inject && self.line.len() > 4 && want_lost {
            let (back, jump) = match variant { 0 => (1, false), 1 => (1, true), _ => (3, false) };
            let lost: Vec<(u64, Txg)> = (0..back).flat_map(|b| {
                let l = &self.line; Self::created_objects(&l[l.len() - 2 - b], &l[l.len() - 1 - b])
            }).collect();
            for o in lost { self.abandoned_objects.insert(o); }
            self.acct(match variant { 0 => "lost_root_one", 1 => "lost_root_one_readable_but_unusable", _ => "lost_root_three" }, 1);
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
        self.acct("recoveries", 1);
        // 挂载：写行、暖机两次空发布，再把队列续做到空。
        let s = self.cur.clone();
        if !self.publish(s, "mount", false) { return; }
        let s = self.cur.clone();
        if !self.publish(s, "mount", false) { return; }
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
        for (list, what) in [(&s.rets, "退役"), (&s.tombs, "墓碑")] {
            for c in list {
                match self.disk[c.slot.unwrap()].as_deref() {
                    Some(Unit { payload: Payload::Ret { id, recs } | Payload::Tomb { id, recs }, .. }) if *id == c.id && *recs == c.recs => {}
                    other => panic!("模型错：回到的那一版的{what}容器 {:?} 不在它的槽里：{other:?}", c.id),
                }
            }
        }
    }

    pub fn rollback(&mut self) -> bool {
        let candidates = self.rollback_candidates();
        if candidates.is_empty() { return true; }
        let i = candidates[self.arng.below(candidates.len() as u64) as usize];
        self.rollback_to(i)
    }

    pub fn rollback_candidates(&self) -> Vec<usize> {
        let n = self.line.len();
        (0..n.saturating_sub(1)).filter(|&i| self.seq - self.line[i].seq + 2 <= self.cfg.ring && self.line[i].txg > 0).collect()
    }

    pub fn rollback_to(&mut self, i: usize) -> bool {
        let target = self.line[i].clone();
        let across = self.structural_seqs.range(target.seq + 1..).next().is_some();
        self.acct("rollbacks", 1);
        if across { self.acct("rollbacks_across_a_structural_publish", 1); }
        let mut next = target.clone();
        // 管理员回退：墓碑树随现行那一版（D23 已定项 14）；退役树两种臂——甲随现行，乙取 R_old（与 inode 树同）。
        next.tombs = self.cur.tombs.clone();
        if self.cfg.cand != Cand::RetB { next.rets = self.cur.rets.clone(); }
        next.watermark = self.cur.watermark.max(target.watermark);
        next.lineage = self.cur.lineage.clone();
        next.lineage.push((self.next_txg, target.txg));
        self.verify_intact(&target);
        self.line.truncate(i + 1);
        let structural: Vec<u64> = self.structural_seqs.range(target.seq + 1..).copied().collect();
        for s in structural { self.structural_seqs.remove(&s); }
        self.note(format!("rollback to txg {}", target.txg));
        if !self.publish(next, "rollback", false) { return false; }
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

    /// 覆盖写：写 u 个单元、size 落在 u 个单元里；`same_size` 时 size 与单元数不变、每个单元换一份新内容。
    pub fn overwrite(&mut self, ino: u64, units: u64, same_size: bool) -> bool {
        let mut next = self.cur.clone();
        let mut rec = next.record(ino).unwrap();
        let (u, size) = if same_size && rec.size > 0 { (units_of(rec.size), rec.size) } else {
            let size = units * PAYLOAD - self.arng.below(PAYLOAD);
            (units, size)
        };
        let anchors: Vec<Anchor> = (0..u).map(|_| Anchor { content: self.new_content(), slot: None }).collect();
        next.data.insert(ino, anchors);
        rec.size = size;
        Self::set_record(&mut next, rec);
        self.publish(next, "overwrite", false)
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

    fn tomb_append(&mut self, next: &mut State, ino: u64, obirth: Txg) {
        let death = self.next_txg;
        self.acct("tombstone_records", 1);
        let cap = self.cfg.tomb_cap;
        let rec = RetRec { who: Ident { num: ino, birth: obirth }, death };
        match next.tombs.last_mut() {
            Some(open) if open.recs.len() < cap => { open.recs.push(rec); open.slot = None; }
            _ => {
                let num = next.tombs.iter().map(|r| r.id.num).max().unwrap_or(0) + 1;
                next.tombs.push(RetC { id: Ident { num, birth: death }, recs: vec![rec], slot: None });
            }
        }
    }

    pub fn unlink(&mut self, ino: u64) -> bool {
        let mut next = self.cur.clone();
        let mut rec = next.record(ino).unwrap();
        rec.nlink = 0;
        Self::set_record(&mut next, rec);
        next.queue = Some(ino);
        if self.cfg.cand != Cand::Fsz { self.tomb_append(&mut next, ino, rec.obirth); }
        self.batch_k = 1 + self.arng.below(3) as usize;
        if !self.publish(next, "intent", false) { return false; }
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
                if !self.publish(next, "batch", false) { return false; }
            } else {
                let obirth = next.record(ino).unwrap().obirth;
                next.data.remove(&ino);
                next.queue = None;
                if self.cfg.cand == Cand::Fsz { self.tomb_append(&mut next, ino, obirth); }
                let kind = self.remove_record(&mut next, ino);
                let tag = match kind { Removed::Plain => "last_plain", Removed::Drop => "last_drop", Removed::Absorb => "last_absorb", Removed::Borrow => "last_borrow" };
                if !self.publish(next, tag, kind != Removed::Plain) { return false; }
            }
        }
        true
    }

    fn retire(&mut self, next: &mut State, who: Ident) {
        let death = self.next_txg;
        self.acct("retirement_records", 1);
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
        self.acct("records_with_data_moved_to_another_leaf", n);
    }

    /// 右邻（合并 / 借的对象）：零条目叶一直留（Persist）时，右邻只认紧邻且非空的；PersistX 跨过零条目叶找下一个非空的。
    fn right_of(&self, next: &State, p: usize) -> Option<usize> {
        match self.cfg.cand {
            Cand::PersistX => (p + 1..next.leaves.len()).find(|&i| !next.leaves[i].recs.is_empty()),
            Cand::Persist => (p + 1 < next.leaves.len() && !next.leaves[p + 1].recs.is_empty()).then_some(p + 1),
            _ => (p + 1 < next.leaves.len()).then_some(p + 1),
        }
    }
    fn left_of(&self, next: &State, p: usize) -> Option<usize> {
        match self.cfg.cand {
            Cand::PersistX => (0..p).rev().find(|&i| !next.leaves[i].recs.is_empty()),
            Cand::Persist => (p > 0 && !next.leaves[p - 1].recs.is_empty()).then(|| p - 1),
            _ => (p > 0).then(|| p - 1),
        }
    }

    /// 容器整片消失（删空、被吸收的右半）：Today / Tsz / Fsz 摘掉不写；RetA / RetB 摘掉并写一条退役记录；Persist 留在叶序里写一版零条目。
    /// 摘掉时右边各片左移、全部重写（照 `crates/` 的位置记法）；零条目叶不挪位置，别的片不重写。
    fn dismiss(&mut self, next: &mut State, p: usize) {
        if self.cfg.cand.keeps_stubs() {
            next.leaves[p].recs.clear();
            next.leaves[p].slot = None;
            self.acct("stub_versions_written", 1);
        } else {
            let gone = next.leaves.remove(p);
            if self.cfg.cand.writes_retirements() { self.retire(next, gone.id); }
            for l in next.leaves.iter_mut().skip(p) { l.slot = None; }
        }
    }

    /// 零条目叶（吸收之后留在叶序里的右半、借时换身份留下的旧身份）的分隔 key 要抬到左邻的最大 key 之上、右邻的分隔 key 之下，
    /// 否则落在它们之间的 key 会路由到这片空叶。严格夹在中间的整数不存在时（右邻分隔 key = 左邻最大 key + 1）没有合法取值：记一次，
    /// 照样写下 左邻最大 key + 1（模型里这一版 I-9.12 为假、路由错）。
    fn raise_the_stub_separator(&mut self, next: &mut State, stub: usize, left: usize) {
        let left_max = next.leaves[left].recs.last().map_or(0, |r| r.ino);
        let candidate = left_max + 1;
        let right_sep = next.leaves.get(stub + 1).map(|l| if self.cfg.borrow == BorrowArm::N1New { l.recs.first().map_or(l.sep, |r| r.ino) } else { l.sep });
        match right_sep {
            Some(rs) if candidate >= rs => self.acct("stub_separator_without_room", 1),
            _ => self.acct("stub_separator_raised_with_room", 1),
        }
        next.leaves[stub].sep = candidate;
    }

    /// unlink 最后一批：删那一条，再照 M2 / M3 均分借的写路径处理那一片。交回这次发布做了什么。
    fn remove_record(&mut self, next: &mut State, ino: u64) -> Removed {
        let cap = self.cfg.leaf_cap;
        let floor = cap / 2;
        let p = next.leaf_of(ino).expect("要删的记录在树里");
        let k = next.leaves[p].recs.binary_search_by_key(&ino, |r| r.ino).unwrap();
        next.leaves[p].recs.remove(k);
        next.leaves[p].slot = None;
        let len = |s: &State, i: usize| s.leaves[i].recs.len();
        if next.leaves[p].recs.is_empty() {
            self.dismiss(next, p);
            self.acct("drops", 1);
            return Removed::Drop;
        }
        if let Some(rn) = self.right_of(next, p) {
            if len(next, p) + len(next, rn) <= cap {
                let right_recs = next.leaves[rn].recs.clone();
                self.moved_with_data(next, &right_recs);
                next.leaves[p].recs.extend(right_recs);
                next.leaves[p].slot = None;
                self.dismiss(next, rn);
                if self.cfg.cand.keeps_stubs() { self.raise_the_stub_separator(next, rn, p); }
                self.acct("absorbs", 1);
                return Removed::Absorb;
            }
        }
        if let Some(ln) = self.left_of(next, p) {
            if len(next, ln) + len(next, p) <= cap {
                let right_recs = next.leaves[p].recs.clone();
                self.moved_with_data(next, &right_recs);
                next.leaves[ln].recs.extend(right_recs);
                next.leaves[ln].slot = None;
                self.dismiss(next, p);
                if self.cfg.cand.keeps_stubs() { self.raise_the_stub_separator(next, p, ln); }
                self.acct("absorbs", 1);
                return Removed::Absorb;
            }
        }
        if self.cfg.borrow != BorrowArm::NoBorrow && len(next, p) < floor {
            if let Some(rn) = self.right_of(next, p) {
                let n = (len(next, rn) - len(next, p)) / 2;
                if n >= 1 {
                    let moved: Vec<Rec> = next.leaves[rn].recs.drain(..n).collect();
                    self.moved_with_data(next, &moved);
                    next.leaves[p].recs.extend(moved);
                    next.leaves[rn].slot = None;
                    let new_min = next.leaves[rn].recs[0].ino;
                    match self.cfg.borrow {
                        BorrowArm::N1New | BorrowArm::N1Raise => { next.leaves[rn].sep = new_min; }
                        BorrowArm::N2 => {
                            let old = next.leaves[rn].id;
                            let old_sep = next.leaves[rn].sep;
                            if self.cfg.cand.writes_retirements() { self.retire(next, old); }
                            next.leaves[rn].id = Ident { num: new_min, birth: self.next_txg };
                            next.leaves[rn].sep = new_min;
                            if self.cfg.cand.keeps_stubs() {
                                // 旧身份的最后一版还装着没挪走的记录：写一版零条目，留在叶序里（落在左片与换了身份的右片之间）。
                                next.leaves.insert(rn, Leaf { id: old, sep: old_sep, recs: Vec::new(), slot: None });
                                self.raise_the_stub_separator(next, rn, p);
                                self.acct("stub_versions_written", 1);
                            }
                        }
                        BorrowArm::NoBorrow => unreachable!(),
                    }
                    self.acct("borrows", 1);
                    return Removed::Borrow;
                }
            }
        }
        Removed::Plain
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

    /// 扫盘重建最新那一版，按读法与真值比。
    /// 读法 S = 今天（R-S）：每组取最新读得出的一版，同号几条取诞生代号最大的那一版供出。
    /// 自己的读法：RetA = S + 退役记录（退役容器只按 P2 择版本，记录按死亡 txg 在不在谱系上滤）；RetB = S + 退役记录（退役容器与叶同一套滤法）；
    /// Persist / PersistX = S（零条目那一版是树持有的活单元，记录算死、槽照收）；Tsz / Fsz = 墓碑读法（按（号, 对象出生代）在所有版本里不收，
    /// 墓碑按死亡 txg 在不在谱系上滤，供出规则：一片叶的版本只有供出了至少一条没被压掉的记录才算活单元）。
    /// 三个名字空间：inode（记录）、data（数据单元按重建出的记录判，读法 Z）、units（分配记录 / 映射按槽号算：被收下的单元的槽与真值引用的槽比）。
    pub fn check(&mut self) {
        let chain = self.chain();
        let at = format!("seq={} txg={} inst={}", self.cur.seq, self.cur.txg, self.cur.inst);
        let in_window = self.cur.queue.is_some();
        let passes: &[(&str, bool, bool)] = if self.cfg.ablate { &[("", true, true), ("_noP2", false, true), ("_noLin", true, false)] } else { &[("", true, true)] };
        let mut judged: Vec<(String, &'static str, u64, u64)> = Vec::new();
        let mut dup_peak = 0u64;
        for (suffix, p2, lin) in passes {
            let (j, d) = self.read_pass(&chain, suffix, *p2, *lin, None, suffix.is_empty());
            judged.extend(j);
            dup_peak = dup_peak.max(d);
        }
        for (name, tree, r, m) in judged { self.stats.judge(&name, tree, r, m, &at, in_window); }
        // 单点证据丢失（两份副本都坏）：丢掉自己的证据单元里最新的那一个（退役 / 墓碑容器里记录最多的，零条目叶里最新写的），重判一遍。每 5 个版本取一次。
        if self.main && self.seq % 5 == 0 {
            if let Some(slot) = self.evidence_slot_to_lose() {
                let (j, _) = self.read_pass(&chain, "", true, true, Some(slot), false);
                for (name, tree, r, m) in j {
                    if name.starts_with("own_") { self.stats.judge(&format!("loss_{}", &name[4..]), tree, r, m, &at, in_window); }
                }
            }
        }
        if self.main {
            self.stats.peak("duplicate_inode_numbers_across_collected_groups_peak", dup_peak);
            self.version_metrics();
        }
    }

    fn evidence_slot_to_lose(&self) -> Option<usize> {
        match self.cfg.cand {
            Cand::RetA | Cand::RetB => self.cur.rets.iter().max_by_key(|c| (c.recs.len(), c.id.num)).and_then(|c| c.slot),
            Cand::Tsz | Cand::Fsz => self.cur.tombs.iter().max_by_key(|c| (c.recs.len(), c.id.num)).and_then(|c| c.slot),
            Cand::Persist | Cand::PersistX => self.cur.leaves.iter().filter(|l| l.recs.is_empty()).filter_map(|l| l.slot).max_by_key(|s| self.disk[*s].as_deref().map_or((0, 0, 0), |u| (u.birth, u.inst, *s))),
            Cand::Today => None,
        }
    }

    /// 一遍扫盘重建：返回 (名字, 名字空间, r, m) 的列表与同号几条的重复数峰值。`skip` 给了就当那个槽的单元（两份副本）读不出。
    fn read_pass(&self, chain: &HashSet<Txg>, suffix: &str, p2: bool, lin: bool, skip: Option<usize>, with_s: bool) -> (Vec<(String, &'static str, u64, u64)>, u64) {
        let truth_inode: HashMap<u64, (Txg, u32, u64)> = self.cur.leaves.iter().flat_map(|l| l.recs.iter())
            .map(|r| (r.ino, (r.obirth, r.nlink, r.size))).collect();
        let truth_data: HashMap<(u64, Txg, u32), u64> = self.cur.data.iter().flat_map(|(ino, a)| {
            let b = truth_inode[ino].0;
            a.iter().enumerate().map(move |(k, x)| ((*ino, b, k as u32), x.content))
        }).collect();
        let truth_slots: HashSet<usize> = self.cur.referenced_slots();
        let own = self.cfg.cand;
        let mut judged: Vec<(String, &'static str, u64, u64)> = Vec::new();
        let mut dup_peak = 0u64;
        let mut leaf_g: HashMap<Ident, (Txg, Inst, usize)> = HashMap::new();
        let mut data_g: HashMap<(u64, Txg, u32), (Txg, Inst, usize, u64)> = HashMap::new();
        // 退役 / 墓碑容器：「随现行」的只按 P2 择版本；「统一」按与叶同一套滤法择版本。
        let mut ret_p2: HashMap<Ident, (Txg, Inst, usize)> = HashMap::new();
        let mut ret_uni: HashMap<Ident, (Txg, Inst, usize)> = HashMap::new();
        let mut tomb_p2: HashMap<Ident, (Txg, Inst, usize)> = HashMap::new();
        for (slot, u) in self.disk.iter().enumerate() {
            if skip == Some(slot) { continue; }
            let Some(u) = u.as_deref() else { continue };
            if p2 && !self.published(u) { continue; }
            let on_chain = chain.contains(&u.birth);
            let key = (u.birth, u.inst);
            match &u.payload {
                Payload::Ret { id, .. } => {
                    let e = ret_p2.entry(*id).or_insert((u.birth, u.inst, slot)); if key > (e.0, e.1) { *e = (u.birth, u.inst, slot); }
                    if !lin || on_chain {
                        let e = ret_uni.entry(*id).or_insert((u.birth, u.inst, slot)); if key > (e.0, e.1) { *e = (u.birth, u.inst, slot); }
                    }
                }
                Payload::Tomb { id, .. } => {
                    let e = tomb_p2.entry(*id).or_insert((u.birth, u.inst, slot)); if key > (e.0, e.1) { *e = (u.birth, u.inst, slot); }
                }
                _ if lin && !on_chain => {}
                Payload::Leaf { id, .. } => { let e = leaf_g.entry(*id).or_insert((u.birth, u.inst, slot)); if key > (e.0, e.1) { *e = (u.birth, u.inst, slot); } }
                Payload::Data { ino, obirth, anchor, content } => {
                    let e = data_g.entry((*ino, *obirth, *anchor)).or_insert((u.birth, u.inst, slot, *content));
                    if key > (e.0, e.1) { *e = (u.birth, u.inst, slot, *content); }
                }
            }
        }
        let tombstoned: HashSet<(u64, Txg)> = tomb_p2.values().flat_map(|(_, _, slot)| {
            match &self.disk[*slot].as_deref().unwrap().payload { Payload::Tomb { recs, .. } => recs.iter().filter(|r| !lin || chain.contains(&r.death)).map(|r| (r.who.num, r.who.birth)).collect::<Vec<_>>(), _ => unreachable!() }
        }).collect();
        let retired_of = |carried: bool| -> HashSet<Ident> {
            let groups = if carried { &ret_p2 } else { &ret_uni };
            let mut s = HashSet::new();
            for (_, _, slot) in groups.values() {
                if let Payload::Ret { recs, .. } = &self.disk[*slot].as_deref().unwrap().payload {
                    for r in recs { if !lin || chain.contains(&r.death) { s.insert(r.who); } }
                }
            }
            s
        };
        // 读法表：(名字, 退役集合, 墓碑读法)
        let mut readers: Vec<(String, HashSet<Ident>, bool)> = Vec::new();
        let own_name = format!("own_{}{}", own.name(), suffix);
        if with_s { readers.push(("S".to_string(), HashSet::new(), false)); }
        match own {
            Cand::Today | Cand::Persist | Cand::PersistX => { if !suffix.is_empty() || own != Cand::Today || skip.is_some() { readers.push((own_name, HashSet::new(), false)); } }
            Cand::RetA => {
                readers.push((own_name, retired_of(true), false));
                // 第二轮攻方腿的反例读法：退役容器与叶按同一套滤法择版本（改法丙），只在 ablate 的段上多跑
                if self.cfg.ablate && suffix.is_empty() && skip.is_none() { readers.push(("own_ret_a_uniform".to_string(), retired_of(false), false)); }
            }
            Cand::RetB => readers.push((own_name, retired_of(false), false)),
            Cand::Tsz | Cand::Fsz => readers.push((own_name, HashSet::new(), true)),
        }
        for (name, retired, tomb_reader) in &readers {
            // 同号几条取供出它的叶版本里 (诞生代号, 实例, 槽) 最大的那一条：平局（只在消融读法里出现）也按槽号定，输出不随哈希次序变。
            let mut inode: HashMap<u64, (Rec, Txg)> = HashMap::new();
            let mut chosen: HashMap<u64, (Txg, Inst, usize)> = HashMap::new();
            let mut supplier: HashMap<u64, Ident> = HashMap::new();
            let mut dups = 0u64;
            for (id, (birth, inst, slot)) in &leaf_g {
                if retired.contains(id) { continue; }
                for rec in self.leaf_recs(*slot) {
                    if *tomb_reader && tombstoned.contains(&(rec.ino, rec.obirth)) { continue; }
                    let key = (*birth, *inst, *slot);
                    match chosen.get(&rec.ino) {
                        Some(k) => { dups += 1; if key > *k { chosen.insert(rec.ino, key); inode.insert(rec.ino, (*rec, *birth)); supplier.insert(rec.ino, *id); } }
                        None => { chosen.insert(rec.ino, key); inode.insert(rec.ino, (*rec, *birth)); supplier.insert(rec.ino, *id); }
                    }
                }
            }
            if name.starts_with("own_") && suffix.is_empty() { dup_peak = dup_peak.max(dups); }
            let r = inode.iter().filter(|(ino, (rec, _))| truth_inode.get(ino) != Some(&(rec.obirth, rec.nlink, rec.size))).count() as u64;
            let m = truth_inode.keys().filter(|ino| !inode.contains_key(ino)).count() as u64;
            judged.push((name.clone(), "inode", r, m));
            // 数据单元：读法 Z（对象出生代相等且锚点 < size 折成的单元数）
            let kept: HashMap<(u64, Txg, u32), (usize, u64)> = data_g.iter().filter(|((ino, ob, anchor), _)| {
                inode.get(ino).is_some_and(|(rec, _)| rec.obirth == *ob && u64::from(*anchor) < units_of(rec.size))
            }).map(|(k, v)| (*k, (v.2, v.3))).collect();
            let r = kept.iter().filter(|(k, (_, c))| truth_data.get(k) != Some(c)).count() as u64;
            let m = truth_data.keys().filter(|k| !kept.contains_key(k)).count() as u64;
            judged.push((name.clone(), "data", r, m));
            // 单元（分配记录 / 映射）：被收下的单元的槽
            let mut collected: HashSet<usize> = HashSet::new();
            let supplying: HashSet<Ident> = supplier.values().copied().collect();
            for (id, (_, _, slot)) in &leaf_g {
                if retired.contains(id) { continue; }
                if *tomb_reader { if supplying.contains(id) { collected.insert(*slot); } } else { collected.insert(*slot); }
            }
            for (slot, _) in kept.values() { collected.insert(*slot); }
            // S 是今天的读法，不认退役 / 墓碑容器这类单元：对着只含叶与数据单元的真值比；自己的读法把退役 / 墓碑容器当单元一并收（乙按统一滤法择版本）。
            let truth_here: HashSet<usize> = if name == "S" {
                let mut t: HashSet<usize> = self.cur.leaves.iter().map(|l| l.slot.unwrap()).collect();
                for a in self.cur.data.values().flatten() { t.insert(a.slot.unwrap()); }
                t
            } else {
                let ret_groups = if own == Cand::RetB { &ret_uni } else { &ret_p2 };
                for (_, _, slot) in ret_groups.values().chain(tomb_p2.values()) { collected.insert(*slot); }
                truth_slots.clone()
            };
            let r = collected.iter().filter(|s| !truth_here.contains(s)).count() as u64;
            let m = truth_here.iter().filter(|s| !collected.contains(s)).count() as u64;
            judged.push((name.clone(), "units", r, m));
        }
        (judged, dup_peak)
    }

    /// 每个版本一次：叶数、零条目叶、I-9.13 / I-9.4 第三句 / I-9.12 与按分隔 key 路由、I-9.18（队列里的号没有墓碑）。
    fn version_metrics(&mut self) {
        let leaves = &self.cur.leaves;
        self.stats.add("versions_judged", 1);
        self.stats.add("leaf_count_sum", leaves.len() as u64);
        self.stats.peak("leaf_count_peak", leaves.len() as u64);
        let stubs = leaves.iter().filter(|l| l.recs.is_empty()).count() as u64;
        self.stats.add("stub_count_sum", stubs);
        self.stats.peak("stub_count_peak", stubs);
        if stubs > 0 { self.stats.add("versions_i913_false", 1); }
        if let Some(ino) = self.cur.queue {
            let has = self.cur.tombs.iter().any(|c| c.recs.iter().any(|r| r.who.num == ino));
            if !has { self.stats.add("versions_i918_false", 1); }
            self.stats.add("versions_in_the_unlink_window", 1);
        }
        // 分隔 key：Today / Ret / Tsz / Fsz / Persist 用容器号；N1New 用孩子这一版的最小 key（零条目叶退回容器号）。
        let seps: Vec<u64> = leaves.iter().map(|l| l.sep).collect();
        let derived = |l: &Leaf| match self.cfg.borrow { BorrowArm::N1New => l.recs.first().map_or(l.id.num, |r| r.ino), _ => l.id.num };
        if leaves.iter().any(|l| l.sep != derived(l)) { self.stats.add("versions_with_a_separator_not_derivable_from_its_leaf", 1); }
        // 两种「从叶重建」的取法（容器号、孩子现在的最小 key）都还原不出存着的分隔 key（第二轮攻方腿同名的那一格）
        if leaves.iter().any(|l| l.sep != l.id.num && l.recs.first().is_some_and(|r| r.ino != l.sep)) { self.stats.add("versions_sep_matches_neither_rebuild_rule", 1); }
        let route = |ino: u64| seps.iter().rposition(|&s| s <= ino);
        let mut i912 = 0u64; let mut misrouted = 0u64; let mut third = 0u64; let mut s1 = 0u64; let mut s2 = 0u64;
        let mut upper: Option<u64> = None;
        for (i, l) in leaves.iter().enumerate() {
            if i > 0 && seps[i] <= seps[i - 1] { i912 += 1; }
            if let Some(first) = l.recs.first() { if seps[i] > first.ino { i912 += 1; } }
            if let Some(u) = upper { if seps[i] <= u { i912 += 1; } }
            if i > 0 && l.id.num <= leaves[i - 1].id.num { s2 += 1; }
            if let Some(first) = l.recs.first() { if first.ino < l.id.num { s1 += 1; } }
            if let (Some(u), true) = (upper, i > 0) { if u >= l.id.num { third += 1; } }
            misrouted += l.recs.iter().filter(|r| route(r.ino) != Some(i)).count() as u64;
            if let Some(last) = l.recs.last() { upper = Some(last.ino); }
        }
        for (name, v) in [("i912", i912), ("misrouted", misrouted), ("i94_third", third), ("i94_first", s1), ("i94_second", s2)] {
            if v > 0 { self.stats.add(&format!("versions_{name}_false"), 1); }
            self.stats.peak(&format!("{name}_peak"), v);
        }
    }
}
