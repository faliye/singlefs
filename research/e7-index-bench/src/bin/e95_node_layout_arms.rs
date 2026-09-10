//! E95：老化的节点布局臂 —— D26（后台整理与放置回收）未定项 4 缺的那条臂。
//!
//! ## 模型（判据与失败条款的权威登记在 kb/experiments/95-老化的节点布局臂.md，跑前写死 2026-09-03）
//!
//! 沿用 E93（老化下的放置与碎片度）的对象 / 槽位模型与两组负载，加两条**格式级**候选臂，
//! 与政策臂在同一套负载、同一组额外写预算下比：
//!
//! | 臂 | 动格式的哪一处 | 机制 |
//! |---|---|---|
//! | 节点组 | 节点头带组身份（G 个 key 相邻的节点构成一个放置与重写单元） | 组内任一节点脏就整组重落一段 |
//! | 代际分离 | 节点头带老化代（自上次重写以来的 checkpoint 数分档） | 放置按代分区，整理只清热区 |
//! | 政策臂（对照） | 不动格式 | E93 的 bump_compact(B) 与 bump_neighbor(R) 原样复用 |
//!
//! **格式臂给最有利的实现**（evidence-discipline.md 稻草人条款）：组身份与老化代当零成本可读，
//! 放置器对它们的利用取最优，两条格式臂的knob都扫一遍取每格最好的那个值。
//!
//! ## 等预算的口径
//!
//! 「额外写倍数 X」在这里是**每 checkpoint 允许的额外写数 = D × (X − 1)**，
//! 四条臂逐轮花掉同一个数（花不完的报出来）。1.5 / 3 / 5 三档对应 extra = 32 / 128 / 256，
//! 而 E93 的 bump_cp_b32 / b128 / b256 逐字报 write_amp = 1.500 / 3.000 / 5.000
//! ⇒ 政策臂那一格与 E93 入库产物是同一个东西，不是新造的。
//!
//! ## 跨装置对照（show-me-test.md「坑在建模口径上，检查要立在装置之间」）
//!
//! 本二进制把 E93 的模型抄了一份。**两套装置算同一个量就要有一条检查逼它们落到同一个数**：
//! `xfixture` 那一批断言把 E93 入库产物 `e93-aging-placement-2026-09-03.out` 里
//! 14 个 aging_median 逐个钉死，对不上判红。
//!
//! ## 它答不了的
//!
//! 1. 计数模型：无耗时轴、无消息缓冲攒批统计、无真机。
//! 2. 两条格式臂是候选不是穷举：别的节点布局形态若被提出，要加臂重跑。
//! 3. 组身份与老化代在真格式里各占几字节、与 D11 的 665 条缓冲抢不抢，模型不算。

use e7_index_bench::Emitter;
use std::collections::BTreeSet;

const L: usize = 8192; // 对象数
const S: usize = 10240; // 槽数（填充 80%）
const G: usize = 64; // 聚簇段槽数
const D: usize = 64; // 每 checkpoint 用户脏对象数
const T: u64 = 2000; // checkpoint 数

/// 等预算三档：额外写倍数 1.5 / 3 / 5 ⇒ 每 checkpoint 额外写 D×(X−1)。
const BUDGETS: [(usize, &str); 3] = [(32, "1.5x"), (128, "3x"), (256, "5x")];

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        let mut s = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0xA076_1D64_78BD_642F);
        if s == 0 {
            s = 0xDEAD_BEEF;
        }
        Rng(s)
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// E93 原样复用的四种政策形态（跨装置对照用）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum E93Policy {
    FirstFit,
    BumpSeg,
    BumpNeighbor(usize),
    BumpCompact(usize),
}
impl E93Policy {
    fn tag(self) -> String {
        match self {
            E93Policy::FirstFit => "first_fit".into(),
            E93Policy::BumpSeg => "bump_seg".into(),
            E93Policy::BumpNeighbor(r) => format!("bump_nb_r{r}"),
            E93Policy::BumpCompact(b) => format!("bump_cp_b{b}"),
        }
    }
}

/// 等预算格上的四条臂。`extra` 是每 checkpoint 的额外写预算。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    /// 政策臂之一：E93 的 bump_compact(B)，B 就是预算本身。
    PolicyCompact(usize),
    /// 政策臂之二：E93 的 bump_neighbor，半径加到预算花完为止。
    PolicyNeighbor(usize),
    /// 格式臂之一：节点组，组大小 g。
    NodeGroup { g: usize, extra: usize },
    /// 格式臂之二：代际分离，热 / 温分档阈值 (h1, h2)。
    GenSep { h1: u64, h2: u64, extra: usize },
}
impl Arm {
    fn tag(self) -> String {
        match self {
            Arm::PolicyCompact(_) => "pol_compact".into(),
            Arm::PolicyNeighbor(_) => "pol_neighbor".into(),
            Arm::NodeGroup { g, .. } => format!("fmt_group_g{g}"),
            Arm::GenSep { h1, h2, .. } => format!("fmt_gensep_{h1}_{h2}"),
        }
    }
    fn family(self) -> &'static str {
        match self {
            Arm::PolicyCompact(_) | Arm::PolicyNeighbor(_) => "policy",
            Arm::NodeGroup { .. } | Arm::GenSep { .. } => "format",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Load {
    Uniform,
    Runs8,
}
impl Load {
    fn tag(self) -> &'static str {
        match self {
            Load::Uniform => "uniform",
            Load::Runs8 => "runs8",
        }
    }
}

struct Sim {
    l: usize,
    s: usize,
    g: usize,
    slot_of: Vec<u32>,
    key_at: Vec<Option<u32>>,
    free: BTreeSet<u32>,
    seg_free: Vec<u16>,
    open_seg: Option<usize>,
    bump: usize,
    /// 增量维护的「断开的邻接对」数；runs = broken + 1（L ≥ 1 时）。
    broken: u64,
    deferred: Vec<u32>,
    user_writes: u64,
    total_writes: u64,
    fallback_allocs: u64,
    sweep: usize,
    /// 代际分离用：每个段被开成哪一代（None = 未定型）。段回空时清掉。
    seg_gen: Vec<Option<u8>>,
    /// 代际分离用：每一代自己的开放段与 bump 位置。
    gen_open: [Option<usize>; 3],
    gen_bump: [usize; 3],
    /// 每个对象上一次被重写的 checkpoint。
    last_write: Vec<u64>,
    /// 每个对象上一次落盘时被判成哪一代。
    obj_gen: Vec<u8>,
}

impl Sim {
    /// 初始布局：key k 住槽 k（顺序创建的形态），与 E93 逐字同一套。
    fn new(l: usize, s: usize, g: usize) -> Sim {
        assert!(l < s && s % g == 0);
        let slot_of: Vec<u32> = (0..l as u32).collect();
        let mut key_at: Vec<Option<u32>> = vec![None; s];
        for k in 0..l {
            key_at[k] = Some(k as u32);
        }
        let free: BTreeSet<u32> = (l as u32..s as u32).collect();
        let mut seg_free = vec![0u16; s / g];
        for slot in l..s {
            seg_free[slot / g] += 1;
        }
        let nseg = s / g;
        Sim {
            l,
            s,
            g,
            slot_of,
            key_at,
            free,
            seg_free,
            open_seg: None,
            bump: 0,
            broken: 0,
            deferred: Vec::new(),
            user_writes: 0,
            total_writes: 0,
            fallback_allocs: 0,
            sweep: 0,
            seg_gen: vec![None; nseg],
            gen_open: [None; 3],
            gen_bump: [0; 3],
            last_write: vec![0; l],
            obj_gen: vec![2; l],
        }
    }

    fn pair_broken(&self, k: usize) -> bool {
        self.slot_of[k] != self.slot_of[k - 1] + 1
    }

    /// 审计：全扫重算 runs。与增量维护不共享累加代码。
    fn audit_runs(&self) -> u64 {
        let mut runs = 1u64;
        for k in 1..self.l {
            if self.slot_of[k] != self.slot_of[k - 1] + 1 {
                runs += 1;
            }
        }
        runs
    }

    fn runs_inc(&self) -> u64 {
        self.broken + 1
    }

    fn move_key(&mut self, key: usize, new_slot: u32) {
        debug_assert!(self.key_at[new_slot as usize].is_none());
        if key >= 1 && self.pair_broken(key) {
            self.broken -= 1;
        }
        if key + 1 < self.l && self.pair_broken(key + 1) {
            self.broken -= 1;
        }
        let old = self.slot_of[key];
        self.key_at[old as usize] = None;
        self.deferred.push(old);
        self.slot_of[key] = new_slot;
        self.key_at[new_slot as usize] = Some(key as u32);
        if key >= 1 && self.pair_broken(key) {
            self.broken += 1;
        }
        if key + 1 < self.l && self.pair_broken(key + 1) {
            self.broken += 1;
        }
        self.total_writes += 1;
    }

    fn alloc_first_fit(&mut self) -> u32 {
        let slot = *self
            .free
            .iter()
            .find(|&&s| self.open_seg != Some(s as usize / self.g))
            .expect("槽用尽：配置违反 D ≤ S−L");
        self.free.remove(&slot);
        self.seg_free[slot as usize / self.g] -= 1;
        slot
    }

    fn open_empty_seg(&mut self) -> Option<usize> {
        (0..self.seg_free.len())
            .find(|&i| self.seg_free[i] as usize == self.g && self.open_seg != Some(i))
    }

    fn alloc_bump(&mut self) -> u32 {
        loop {
            if let Some(seg) = self.open_seg {
                if self.bump < self.g {
                    let slot = (seg * self.g + self.bump) as u32;
                    self.bump += 1;
                    debug_assert!(self.free.contains(&slot));
                    self.free.remove(&slot);
                    self.seg_free[seg] -= 1;
                    return slot;
                }
                self.open_seg = None;
            }
            match self.open_empty_seg() {
                Some(seg) => {
                    self.open_seg = Some(seg);
                    self.bump = 0;
                }
                None => {
                    self.fallback_allocs += 1;
                    return self.alloc_first_fit();
                }
            }
        }
    }

    fn find_free_run(&self, len: usize) -> Option<u32> {
        let mut start = 0u32;
        let mut count = 0usize;
        let mut prev: Option<u32> = None;
        for &s in &self.free {
            if self.open_seg == Some(s as usize / self.g) {
                prev = None;
                count = 0;
                continue;
            }
            match prev {
                Some(p) if s == p + 1 => count += 1,
                _ => {
                    start = s;
                    count = 1;
                }
            }
            prev = Some(s);
            if count >= len {
                return Some(start);
            }
        }
        None
    }

    fn take_slot(&mut self, slot: u32) {
        let removed = self.free.remove(&slot);
        debug_assert!(removed);
        self.seg_free[slot as usize / self.g] -= 1;
    }

    fn place_run(&mut self, keys: &[usize]) {
        let g = self.g;
        let owned: Vec<usize> = keys.to_vec();
        for chunk in owned.chunks(g) {
            self.place_chunk(chunk);
        }
    }

    fn place_chunk(&mut self, keys: &[usize]) {
        let len = keys.len();
        if let Some(seg) = self.open_seg {
            if self.g - self.bump >= len {
                for &k in keys {
                    let slot = (seg * self.g + self.bump) as u32;
                    self.bump += 1;
                    self.take_slot(slot);
                    self.move_key(k, slot);
                }
                return;
            }
        }
        if let Some(seg) = self.open_empty_seg() {
            self.open_seg = Some(seg);
            self.bump = 0;
            for &k in keys {
                let slot = (seg * self.g + self.bump) as u32;
                self.bump += 1;
                self.take_slot(slot);
                self.move_key(k, slot);
            }
            return;
        }
        if let Some(start) = self.find_free_run(len) {
            for (i, &k) in keys.iter().enumerate() {
                let slot = start + i as u32;
                self.take_slot(slot);
                self.move_key(k, slot);
            }
            return;
        }
        for &k in keys {
            self.fallback_allocs += 1;
            let slot = self.alloc_first_fit();
            self.move_key(k, slot);
        }
    }

    // ---- 代际分离臂专用的放置面 ----

    /// 找一个能开给第 `gen` 代的全空段（全空段没有代，谁开归谁）。
    fn open_empty_seg_for(&self, gen: u8) -> Option<usize> {
        (0..self.seg_free.len()).find(|&i| {
            self.seg_free[i] as usize == self.g
                && self.gen_open[0] != Some(i)
                && self.gen_open[1] != Some(i)
                && self.gen_open[2] != Some(i)
                && self.seg_gen[i].map_or(true, |g| g == gen)
        })
    }

    /// 按代分区落一批 key（同一代、按 key 序）。热与冷永不落进同一个段。
    fn place_gen_run(&mut self, gen: u8, keys: &[usize]) {
        let gi = gen as usize;
        for &k in keys {
            loop {
                if let Some(seg) = self.gen_open[gi] {
                    if self.gen_bump[gi] < self.g {
                        let slot = (seg * self.g + self.gen_bump[gi]) as u32;
                        self.gen_bump[gi] += 1;
                        debug_assert!(self.free.contains(&slot));
                        self.take_slot(slot);
                        self.move_key(k, slot);
                        break;
                    }
                    self.gen_open[gi] = None;
                }
                match self.open_empty_seg_for(gen) {
                    Some(seg) => {
                        self.gen_open[gi] = Some(seg);
                        self.gen_bump[gi] = 0;
                        self.seg_gen[seg] = Some(gen);
                    }
                    None => {
                        // 无空段：回落逐槽分配，计数（与 E93 的回落口径同一个数）
                        self.fallback_allocs += 1;
                        let slot = self.alloc_first_fit_gen();
                        self.move_key(k, slot);
                        break;
                    }
                }
            }
        }
    }

    /// 代际分离臂的回落：三代的开放段都是保留区，first_fit 不许拿。
    fn alloc_first_fit_gen(&mut self) -> u32 {
        let slot = *self
            .free
            .iter()
            .find(|&&s| {
                let seg = s as usize / self.g;
                self.gen_open[0] != Some(seg)
                    && self.gen_open[1] != Some(seg)
                    && self.gen_open[2] != Some(seg)
            })
            .expect("槽用尽：配置违反 D ≤ S−L");
        self.free.remove(&slot);
        self.seg_free[slot as usize / self.g] -= 1;
        slot
    }

    /// checkpoint 收尾：defer 的槽此刻才真正可复用（D16 新规则 2）。
    fn end_checkpoint(&mut self) {
        for slot in std::mem::take(&mut self.deferred) {
            self.free.insert(slot);
            self.seg_free[slot as usize / self.g] += 1;
        }
        // 回空的段丢掉代身份（否则代分区会把空段永久锁给某一代）
        for i in 0..self.seg_free.len() {
            if self.seg_free[i] as usize == self.g
                && self.gen_open[0] != Some(i)
                && self.gen_open[1] != Some(i)
                && self.gen_open[2] != Some(i)
            {
                self.seg_gen[i] = None;
            }
        }
    }

    fn occupied(&self) -> usize {
        self.key_at.iter().filter(|k| k.is_some()).count()
    }
}

/// 选负载脏集（去重、升序）。与 E93 逐字同一套。
fn dirty_set(load: Load, rng: &mut Rng, l: usize, d: usize) -> Vec<usize> {
    let mut set = BTreeSet::new();
    match load {
        Load::Uniform => {
            while set.len() < d {
                set.insert(rng.below(l));
            }
        }
        Load::Runs8 => {
            while set.len() < d {
                let start = rng.below(l);
                for k in start..(start + 8).min(l) {
                    if set.len() < d {
                        set.insert(k);
                    }
                }
            }
        }
    }
    set.into_iter().collect()
}

/// E93 的 bump_neighbor：把脏 key 段向两侧各扩 ≤R 个干净邻居。
fn extend_neighbors(dirty: &[usize], r: usize, l: usize) -> (Vec<usize>, u64) {
    let mut set: BTreeSet<usize> = dirty.iter().copied().collect();
    let mut extra = 0u64;
    let mut runs: Vec<(usize, usize)> = Vec::new();
    for &k in dirty {
        match runs.last_mut() {
            Some((_, b)) if *b + 1 == k => *b = k,
            _ => runs.push((k, k)),
        }
    }
    for (a, b) in runs {
        for k in (a.saturating_sub(r)..a).chain(b + 1..(b + 1 + r).min(l)) {
            if set.insert(k) {
                extra += 1;
            }
        }
    }
    (set.into_iter().collect(), extra)
}

/// 预算封顶的邻居扩展：半径从 1 起一圈圈加，加到预算花完为止。
/// **与 E93 的 bump_neighbor 差在封顶**——不封顶就落不到等预算格上。
fn extend_neighbors_budgeted(dirty: &[usize], budget: usize, l: usize) -> Vec<usize> {
    let mut set: BTreeSet<usize> = dirty.iter().copied().collect();
    let mut spent = 0usize;
    let mut runs: Vec<(usize, usize)> = Vec::new();
    for &k in dirty {
        match runs.last_mut() {
            Some((_, b)) if *b + 1 == k => *b = k,
            _ => runs.push((k, k)),
        }
    }
    let mut r = 1usize;
    while spent < budget && r <= l {
        let mut grew = false;
        for &(a, b) in &runs {
            for k in [a.checked_sub(r), Some(b + r)].into_iter().flatten() {
                if k < l && spent < budget && set.insert(k) {
                    spent += 1;
                    grew = true;
                }
            }
        }
        if !grew {
            break;
        }
        r += 1;
    }
    set.into_iter().collect()
}

struct Outcome {
    runs_final: u64,
    write_amp: f64,
    fallback_pct: f64,
    extra_spent_pct: f64,
}

/// E93 的四条政策臂，逐字照抄它的 run_arm——跨装置对照就靠这一段。
fn run_e93_arm(policy: E93Policy, load: Load, seed: u64, t_max: u64, sample_every: u64) -> Outcome {
    let mut sim = Sim::new(L, S, G);
    let mut rng = Rng::new(seed);
    for t in 1..=t_max {
        let dirty = dirty_set(load, &mut rng, L, D);
        sim.user_writes += dirty.len() as u64;
        let (batch, _extra) = match policy {
            E93Policy::BumpNeighbor(r) => extend_neighbors(&dirty, r, L),
            _ => (dirty, 0),
        };
        match policy {
            E93Policy::FirstFit => {
                for &key in &batch {
                    let slot = sim.alloc_first_fit();
                    sim.move_key(key, slot);
                }
            }
            E93Policy::BumpSeg | E93Policy::BumpCompact(_) => {
                for &key in &batch {
                    let slot = sim.alloc_bump();
                    sim.move_key(key, slot);
                }
            }
            E93Policy::BumpNeighbor(_) => {
                let mut i = 0;
                while i < batch.len() {
                    let mut j = i + 1;
                    while j < batch.len() && batch[j] == batch[j - 1] + 1 {
                        j += 1;
                    }
                    let run: Vec<usize> = batch[i..j].to_vec();
                    sim.place_run(&run);
                    i = j;
                }
            }
        }
        if let E93Policy::BumpCompact(b) = policy {
            let batch_set: BTreeSet<usize> = batch.iter().copied().collect();
            let mut sweep_keys = Vec::with_capacity(b);
            while sweep_keys.len() < b {
                let key = sim.sweep;
                sim.sweep = (sim.sweep + 1) % L;
                if !batch_set.contains(&key) {
                    sweep_keys.push(key);
                }
            }
            sim.place_run(&sweep_keys);
        }
        sim.end_checkpoint();
        if t % sample_every == 0 {
            assert_eq!(sim.runs_inc(), sim.audit_runs(), "t={t} 增量与审计分叉");
            assert_eq!(sim.occupied(), sim.l);
            assert_eq!(sim.free.len() + sim.l, sim.s);
        }
    }
    Outcome {
        runs_final: sim.runs_inc(),
        write_amp: sim.total_writes as f64 / sim.user_writes as f64,
        fallback_pct: 100.0 * sim.fallback_allocs as f64 / sim.total_writes as f64,
        extra_spent_pct: 0.0,
    }
}

/// 老化代分档：自上次重写以来的 checkpoint 数落在哪一档。0 = 热，1 = 温，2 = 冷。
fn age_gen(age: u64, h1: u64, h2: u64) -> u8 {
    if age < h1 {
        0
    } else if age < h2 {
        1
    } else {
        2
    }
}

fn run_arm(arm: Arm, load: Load, seed: u64, t_max: u64, sample_every: u64) -> Outcome {
    let mut sim = Sim::new(L, S, G);
    let mut rng = Rng::new(seed);
    let mut budget_total = 0u64;
    let mut budget_spent = 0u64;
    let mut group_cursor = 0usize;
    for t in 1..=t_max {
        let dirty = dirty_set(load, &mut rng, L, D);
        sim.user_writes += dirty.len() as u64;
        let before = sim.total_writes;
        match arm {
            Arm::PolicyCompact(extra) => {
                budget_total += extra as u64;
                for &key in &dirty {
                    let slot = sim.alloc_bump();
                    sim.move_key(key, slot);
                }
                let dirty_set_: BTreeSet<usize> = dirty.iter().copied().collect();
                let mut sweep_keys = Vec::with_capacity(extra);
                while sweep_keys.len() < extra {
                    let key = sim.sweep;
                    sim.sweep = (sim.sweep + 1) % L;
                    if !dirty_set_.contains(&key) {
                        sweep_keys.push(key);
                    }
                }
                sim.place_run(&sweep_keys);
            }
            Arm::PolicyNeighbor(extra) => {
                budget_total += extra as u64;
                let batch = extend_neighbors_budgeted(&dirty, extra, L);
                let mut i = 0;
                while i < batch.len() {
                    let mut j = i + 1;
                    while j < batch.len() && batch[j] == batch[j - 1] + 1 {
                        j += 1;
                    }
                    let run: Vec<usize> = batch[i..j].to_vec();
                    sim.place_run(&run);
                    i = j;
                }
            }
            Arm::NodeGroup { g, extra } => {
                budget_total += extra as u64;
                step_node_group(&mut sim, &dirty, g, extra, &mut group_cursor);
            }
            Arm::GenSep { h1, h2, extra } => {
                budget_total += extra as u64;
                step_gen_sep(&mut sim, &dirty, t, h1, h2, extra);
            }
        }
        budget_spent += (sim.total_writes - before).saturating_sub(dirty.len() as u64);
        sim.end_checkpoint();
        if t % sample_every == 0 {
            assert_eq!(sim.runs_inc(), sim.audit_runs(), "t={t} 增量与审计分叉");
            assert_eq!(sim.occupied(), sim.l);
            assert_eq!(sim.free.len() + sim.l, sim.s);
        }
    }
    Outcome {
        runs_final: sim.runs_inc(),
        write_amp: sim.total_writes as f64 / sim.user_writes as f64,
        fallback_pct: 100.0 * sim.fallback_allocs as f64 / sim.total_writes as f64,
        extra_spent_pct: 100.0 * budget_spent as f64 / budget_total as f64,
    }
}

/// 节点组臂的一个 checkpoint。抽成函数是为了让断言够得着——
/// 留在 run_arm 里的时候，一条「组只落脏的那几个」的变异一个测试都不会红。
fn step_node_group(
    sim: &mut Sim,
    dirty: &[usize],
    g: usize,
    extra: usize,
    group_cursor: &mut usize,
) {
    let l = sim.l;
    let dirty_set_: BTreeSet<usize> = dirty.iter().copied().collect();
    let mut remaining = extra;
    let mut written: BTreeSet<usize> = BTreeSet::new();
    // 被脏集碰到的组，按 key 序：预算够就整组重落，不够就只落脏的那几个
    let mut touched: Vec<usize> = dirty.iter().map(|&k| k / g).collect();
    touched.dedup();
    for grp in touched {
        let lo = grp * g;
        let hi = ((grp + 1) * g).min(l);
        let clean = (lo..hi).filter(|k| !dirty_set_.contains(k)).count();
        let keys: Vec<usize> = if clean <= remaining {
            remaining -= clean;
            (lo..hi).collect()
        } else {
            (lo..hi).filter(|k| dirty_set_.contains(k)).collect()
        };
        for &k in &keys {
            written.insert(k);
        }
        sim.place_run(&keys);
    }
    // 预算还有剩：按组轮转，主动整组重落（组是放置单位，所以整组落）
    let groups = l.div_ceil(g);
    let mut scanned = 0usize;
    while remaining >= g && scanned < groups {
        let lo = *group_cursor * g;
        let hi = ((*group_cursor + 1) * g).min(l);
        *group_cursor = (*group_cursor + 1) % groups;
        scanned += 1;
        if (lo..hi).any(|k| written.contains(&k)) {
            continue;
        }
        let keys: Vec<usize> = (lo..hi).collect();
        remaining -= keys.len();
        for &k in &keys {
            written.insert(k);
        }
        sim.place_run(&keys);
    }
}

/// 代际分离臂的一个 checkpoint。抽出来的理由同上。
fn step_gen_sep(sim: &mut Sim, dirty: &[usize], t: u64, h1: u64, h2: u64, extra: usize) {
    let l = sim.l;
    // 脏对象：按「自上次重写以来多久」定代，同代同段、按 key 序
    let mut by_gen: [Vec<usize>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    for &k in dirty {
        let gen = age_gen(t - sim.last_write[k], h1, h2);
        by_gen[gen as usize].push(k);
        sim.obj_gen[k] = gen;
        sim.last_write[k] = t;
    }
    for gen in 0..3u8 {
        let keys = std::mem::take(&mut by_gen[gen as usize]);
        sim.place_gen_run(gen, &keys);
    }
    // 整理只清热区：按 key 序轮转，只挑现在住在热代里的对象
    let dirty_set_: BTreeSet<usize> = dirty.iter().copied().collect();
    let mut sweep_keys = Vec::with_capacity(extra);
    let mut looked = 0usize;
    while sweep_keys.len() < extra && looked < l {
        let key = sim.sweep;
        sim.sweep = (sim.sweep + 1) % l;
        looked += 1;
        if !dirty_set_.contains(&key) && sim.obj_gen[key] == 0 {
            sweep_keys.push(key);
        }
    }
    for &k in &sweep_keys {
        sim.last_write[k] = t;
    }
    sim.place_gen_run(0, &sweep_keys);
}

/// 判据 3：测量的阳性对照。强制放置（绕过政策）走增量路径，审计路径复核。
fn measurement_control(sim: &mut Sim) -> (u64, u64, u64, u64) {
    let l = sim.l;
    for k in 0..l {
        let target = if k < l / 2 { 2 * k as u32 } else { (2 * (k - l / 2) + 1) as u32 };
        force_to(sim, k, target);
    }
    let scatter = (sim.runs_inc(), sim.audit_runs());
    for k in 0..l {
        force_to(sim, k, k as u32);
    }
    let compact = (sim.runs_inc(), sim.audit_runs());
    (scatter.0, scatter.1, compact.0, compact.1)
}

fn force_to(sim: &mut Sim, key: usize, target: u32) {
    if sim.slot_of[key] == target {
        return;
    }
    if let Some(other) = sim.key_at[target as usize] {
        let spare = *sim.free.iter().next().expect("强制放置需要至少一个空槽");
        sim.free.remove(&spare);
        sim.seg_free[spare as usize / sim.g] -= 1;
        sim.move_key(other as usize, spare);
    }
    if sim.free.remove(&target) {
        sim.seg_free[target as usize / sim.g] -= 1;
    } else {
        let pos = sim.deferred.iter().position(|&s| s == target).expect("目标槽既不空也不在 defer");
        sim.deferred.swap_remove(pos);
    }
    sim.move_key(key, target);
    sim.end_checkpoint();
}

/// E93 入库产物 `research/results/e93-aging-placement-2026-09-03.out` 里的 14 个 aging_median，
/// 逐字钉死。**这是跨装置的那道闸**：本二进制抄了一份 E93 的模型，
/// 两份算同一个量，对不上就判红（show-me-test.md「坑在建模口径上，检查要立在装置之间」）。
const E93_MEDIANS: [(&str, &str, u64); 14] = [
    ("first_fit", "uniform", 8186),
    ("first_fit", "runs8", 7997),
    ("bump_seg", "uniform", 8186),
    ("bump_seg", "runs8", 7312),
    ("bump_nb_r1", "uniform", 5497),
    ("bump_nb_r1", "runs8", 5181),
    ("bump_nb_r2", "uniform", 5460),
    ("bump_nb_r2", "runs8", 5111),
    ("bump_cp_b32", "uniform", 7502),
    ("bump_cp_b32", "runs8", 6073),
    ("bump_cp_b128", "uniform", 6095),
    ("bump_cp_b128", "runs8", 3922),
    ("bump_cp_b256", "uniform", 4791),
    ("bump_cp_b256", "runs8", 1339),
];

fn median5(mut v: Vec<u64>) -> u64 {
    v.sort_unstable();
    v[2]
}

fn main() {
    let mut em = Emitter::new();
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=config l={L} s={S} g={G} d={D} t={T} model=counting file_ops=0 seeds=5 budgets=32,128,256"
        ))
    );

    // 等预算格上的臂：政策两条 + 格式两条（格式臂的 knob 各扫三个值，取每格最好的）
    let mut arms: Vec<Arm> = Vec::new();
    for (extra, _) in BUDGETS {
        arms.push(Arm::PolicyCompact(extra));
        arms.push(Arm::PolicyNeighbor(extra));
        for g in [2usize, 8, 32] {
            arms.push(Arm::NodeGroup { g, extra });
        }
        for (h1, h2) in [(16u64, 128u64), (64, 512), (256, 1024)] {
            arms.push(Arm::GenSep { h1, h2, extra });
        }
    }

    // 判据 3：阳性对照**每条臂都过闸**（test-discipline.md「阳性对照必须对每一条被测的臂都跑」）
    let mut control_arms: Vec<String> = Vec::new();
    for &arm in &arms {
        let key = format!("{}", arm.tag());
        if control_arms.contains(&key) {
            continue;
        }
        control_arms.push(key.clone());
        let mut sim = Sim::new(L, S, G);
        let (si, sa, ci, ca) = measurement_control(&mut sim);
        assert_eq!((si, sa), (L as u64, L as u64), "{:?} 全隔离布局必须报 runs=L", arm);
        assert_eq!((ci, ca), (1, 1), "{:?} 连续布局必须报 runs=1", arm);
        println!(
            "{}",
            em.emit_raw(&format!("name=control arm={key} scatter_runs={si} compact_runs={ci}"))
        );
    }
    for policy in [
        E93Policy::FirstFit,
        E93Policy::BumpSeg,
        E93Policy::BumpNeighbor(1),
        E93Policy::BumpNeighbor(2),
        E93Policy::BumpCompact(32),
        E93Policy::BumpCompact(128),
        E93Policy::BumpCompact(256),
    ] {
        let mut sim = Sim::new(L, S, G);
        let (si, sa, ci, ca) = measurement_control(&mut sim);
        assert_eq!((si, sa), (L as u64, L as u64), "{:?} 全隔离布局必须报 runs=L", policy);
        assert_eq!((ci, ca), (1, 1), "{:?} 连续布局必须报 runs=1", policy);
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=control arm=e93_{} scatter_runs={si} compact_runs={ci}",
                policy.tag()
            ))
        );
    }

    // 跨装置闸：E93 的 14 个 aging_median 必须逐个复现
    for (arm_tag, load_tag, want) in E93_MEDIANS {
        let policy = match arm_tag {
            "first_fit" => E93Policy::FirstFit,
            "bump_seg" => E93Policy::BumpSeg,
            "bump_nb_r1" => E93Policy::BumpNeighbor(1),
            "bump_nb_r2" => E93Policy::BumpNeighbor(2),
            "bump_cp_b32" => E93Policy::BumpCompact(32),
            "bump_cp_b128" => E93Policy::BumpCompact(128),
            "bump_cp_b256" => E93Policy::BumpCompact(256),
            other => panic!("跨装置表里有认不出的臂：{other}"),
        };
        let load = match load_tag {
            "uniform" => Load::Uniform,
            "runs8" => Load::Runs8,
            other => panic!("跨装置表里有认不出的负载：{other}"),
        };
        let got = median5((0..5u64).map(|s| run_e93_arm(policy, load, s, T, 250).runs_final).collect());
        assert_eq!(
            got, want,
            "跨装置对照失败：{arm_tag}/{load_tag} 本装置报 {got}，E93 入库产物里是 {want}"
        );
        println!(
            "{}",
            em.emit_raw(&format!(
                "name=xfixture arm={arm_tag} load={load_tag} median={got} e93_stored={want}"
            ))
        );
    }

    // 判据 4：等预算格
    let mut grid: Vec<(usize, String, &'static str, u64, f64, f64, f64)> = Vec::new();
    for &arm in &arms {
        let extra = match arm {
            Arm::PolicyCompact(e) | Arm::PolicyNeighbor(e) => e,
            Arm::NodeGroup { extra, .. } | Arm::GenSep { extra, .. } => extra,
        };
        for load in [Load::Uniform, Load::Runs8] {
            let mut finals = Vec::new();
            let mut amp = 0.0;
            let mut fb = 0.0;
            let mut spent = 0.0;
            for seed in 0..5u64 {
                let o = run_arm(arm, load, seed, T, 250);
                if seed == 0 {
                    amp = o.write_amp;
                    fb = o.fallback_pct;
                    spent = o.extra_spent_pct;
                }
                finals.push(o.runs_final);
            }
            let mid = median5(finals);
            // **绝对值闸**：等预算格上任何一条臂都不许超预算。
            // 只有互比（「格式臂比政策臂低」）测不出「所有臂一起多花了钱」。
            let cap = (D + extra) as f64 / D as f64;
            assert!(
                amp <= cap + 1e-9,
                "{} 在预算 {extra} 上花超了：write_amp={amp:.6} > {cap:.6}",
                arm.tag()
            );
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=grid budget={extra} arm={} family={} load={} runs_median={mid} write_amp={amp:.3} budget_spent_pct={spent:.1} fallback_pct={fb:.1}",
                    arm.tag(),
                    arm.family(),
                    load.tag()
                ))
            );
            grid.push((extra, arm.tag(), load.tag(), mid, amp, spent, fb));
        }
    }

    // 判据 5：格式臂在任一等预算格上能不能把 runs 压到政策臂的一半以下
    let mut any_halved = false;
    for (extra, _) in BUDGETS {
        for load in ["uniform", "runs8"] {
            let best_policy = grid
                .iter()
                .filter(|g| g.0 == extra && g.2 == load && g.1.starts_with("pol_"))
                .map(|g| g.3)
                .min()
                .expect("每格必须有政策臂");
            let (best_fmt_tag, best_fmt) = grid
                .iter()
                .filter(|g| g.0 == extra && g.2 == load && g.1.starts_with("fmt_"))
                .map(|g| (g.1.clone(), g.3))
                .min_by_key(|(_, v)| *v)
                .expect("每格必须有格式臂");
            let halved = best_fmt * 2 < best_policy;
            any_halved |= halved;
            println!(
                "{}",
                em.emit_raw(&format!(
                    "name=verdict budget={extra} load={load} best_policy={best_policy} best_format={best_fmt} best_format_arm={best_fmt_tag} format_halves_policy={halved}"
                ))
            );
        }
    }
    println!(
        "{}",
        em.emit_raw(&format!(
            "name=answer any_cell_format_halves_policy={any_halved} criterion=E95_5"
        ))
    );
    println!("{}", em.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **判据 1 手算锚点**：8 对象、10 槽、组大小 2。
    /// 初始 key k 住槽 k ⇒ runs = 1。脏 key 3 ⇒ 组 1 是 {2,3}，整组重落。
    /// 组 {2,3} 落到最低的连续空 run：槽 8 与 9（0..7 满，8/9 空）
    /// ⇒ 布局 0,1,[8],[9],4,5,6,7 ⇒ 断在 (1,2)、(3,4) ⇒ runs = 3。
    #[test]
    fn hand_case_group_rewrite_runs() {
        let mut sim = Sim::new(8, 10, 2);
        assert_eq!(sim.runs_inc(), 1);
        assert_eq!(sim.audit_runs(), 1);
        sim.place_run(&[2, 3]);
        assert_eq!(sim.slot_of[2], 8, "组 {{2,3}} 必须整组落到空 run 的头上");
        assert_eq!(sim.slot_of[3], 9);
        assert_eq!(sim.runs_inc(), 3, "手算锚点：断在 (1,2) 与 (3,4)");
        assert_eq!(sim.audit_runs(), 3);
    }

    /// 手算锚点之二：只重落脏的那一个（组大小 1 的退化形态）⇒ runs = 3，
    /// 而它挪走之后 (2,3) 与 (3,4) 两对都断 ⇒ 与整组重落的 runs 相同但写了一次。
    /// **这两条一起钉的是「组重落买到的不是当轮的 runs，是后续的稳定性」。**
    #[test]
    fn hand_case_single_rewrite_runs() {
        let mut sim = Sim::new(8, 10, 2);
        sim.place_run(&[3]);
        assert_eq!(sim.slot_of[3], 8);
        assert_eq!(sim.runs_inc(), 3);
        assert_eq!(sim.audit_runs(), 3);
        assert_eq!(sim.total_writes, 1, "只写了脏的那一个");
    }

    /// 判据 2 守恒：增量 runs 与审计 runs 逐点相等，占用恒 = L，空 + 占 = S。
    #[test]
    fn conservation_holds_on_every_arm() {
        for arm in [
            Arm::PolicyCompact(32),
            Arm::PolicyNeighbor(32),
            Arm::NodeGroup { g: 8, extra: 32 },
            Arm::GenSep { h1: 64, h2: 512, extra: 32 },
        ] {
            let o = run_arm(arm, Load::Uniform, 0, 200, 25);
            assert!(o.runs_final >= 1 && o.runs_final <= L as u64);
        }
    }

    /// 判据 3 阳性对照：测量管道分得出全隔离与全连续。
    #[test]
    fn measurement_control_discriminates() {
        let mut sim = Sim::new(L, S, G);
        let (si, sa, ci, ca) = measurement_control(&mut sim);
        assert_eq!((si, sa), (L as u64, L as u64));
        assert_eq!((ci, ca), (1, 1));
    }

    /// 跨装置闸自己要能红：E93 入库产物里的一个数被改掉时，断言必须不成立。
    #[test]
    fn xfixture_gate_is_not_a_rubber_stamp() {
        let got = median5((0..5u64).map(|s| run_e93_arm(E93Policy::BumpCompact(256), Load::Runs8, s, T, 250).runs_final).collect());
        assert_eq!(got, 1339, "本装置复现 E93 的 bump_cp_b256/runs8");
        assert_ne!(got, 1340, "若这条也成立，说明断言根本没在比");
    }

    /// **跨装置闸的本体**：E93 入库产物里的 14 个 aging_median 逐行复现。
    /// ⚠️ 这条必须遍历整张表——只钉一行的话，改表里别的行一个测试都不会红（实测过）。
    #[test]
    fn xfixture_reproduces_every_stored_e93_median() {
        for (arm_tag, load_tag, want) in E93_MEDIANS {
            let policy = match arm_tag {
                "first_fit" => E93Policy::FirstFit,
                "bump_seg" => E93Policy::BumpSeg,
                "bump_nb_r1" => E93Policy::BumpNeighbor(1),
                "bump_nb_r2" => E93Policy::BumpNeighbor(2),
                "bump_cp_b32" => E93Policy::BumpCompact(32),
                "bump_cp_b128" => E93Policy::BumpCompact(128),
                "bump_cp_b256" => E93Policy::BumpCompact(256),
                other => panic!("跨装置表里有认不出的臂：{other}"),
            };
            let load = match load_tag {
                "uniform" => Load::Uniform,
                "runs8" => Load::Runs8,
                other => panic!("跨装置表里有认不出的负载：{other}"),
            };
            let got =
                median5((0..5u64).map(|s| run_e93_arm(policy, load, s, T, 250).runs_final).collect());
            assert_eq!(got, want, "跨装置对照失败：{arm_tag}/{load_tag}");
        }
    }

    /// **节点组臂的绝对值锚点**：脏一个 key，整组必须落成一段连续物理槽。
    /// 这一条钉的是「组是放置单位」本身，不是任何臂间比较。
    #[test]
    fn node_group_step_relands_whole_group_contiguously() {
        let mut sim = Sim::new(32, 64, 4);
        let mut cursor = 0usize;
        // 组大小 4、预算 3（够整组重落一次：一个脏 + 三个干净）
        step_node_group(&mut sim, &[5], 4, 3, &mut cursor);
        let base = sim.slot_of[4];
        for (i, k) in (4..8).enumerate() {
            assert_eq!(sim.slot_of[k], base + i as u32, "组 {{4..7}} 必须整组连续");
        }
        assert_eq!(sim.total_writes, 4, "整组四个对象各写一次");
        // 预算 0：只落脏的那一个，组不整体重落
        let mut sim2 = Sim::new(32, 64, 4);
        let mut cursor2 = 0usize;
        step_node_group(&mut sim2, &[5], 4, 0, &mut cursor2);
        assert_eq!(sim2.total_writes, 1, "预算 0 时只写脏的那一个");
        assert_eq!(sim2.slot_of[4], 4, "干净的邻居不许动");
    }

    /// 预算有剩时按组轮转主动重落，且轮转游标真的在走。
    #[test]
    fn node_group_step_spends_leftover_budget_round_robin() {
        let mut sim = Sim::new(32, 64, 4);
        let mut cursor = 0usize;
        // 脏 key 20 所在的组 5 要 3 个干净的；预算 11 ⇒ 剩 8 ⇒ 再整落两个组
        step_node_group(&mut sim, &[20], 4, 11, &mut cursor);
        assert_eq!(sim.total_writes, 4 + 8, "一个脏组 + 两个轮转组");
        assert_eq!(cursor, 2, "游标走过两个组");
    }

    /// 同一个组里两个脏 key 只算一个组。
    /// ⚠️ 这条是补的**敏感取样点**：只脏一个 key 时 dedup 去不去都一样
    /// （mutation-sampling.md「取样点不敏感」那一类，不是等价变异）。
    #[test]
    fn node_group_step_counts_each_group_once() {
        let mut sim = Sim::new(32, 64, 4);
        let mut cursor = 0usize;
        // key 4 与 5 同属组 1 ⇒ 组只该被整落一次，写 4 次不是 8 次
        step_node_group(&mut sim, &[4, 5], 4, 2, &mut cursor);
        assert_eq!(sim.total_writes, 4, "同组的两个脏 key 只该让这个组整落一次");
    }

    /// 代际分离的整理**只清热区**：住在温 / 冷代里的对象不许被扫进整理批次。
    #[test]
    fn gen_sep_sweep_only_takes_hot_objects() {
        let mut sim = Sim::new(16, 64, 4);
        // 全部对象初值是冷（obj_gen = 2）⇒ 整理一个都挑不到
        let before = sim.total_writes;
        step_gen_sep(&mut sim, &[0], 1, 16, 128, 8);
        assert_eq!(sim.total_writes, before + 1, "只有那个脏 key 被写，整理挑不到热对象");
        // 把 5..9 标成热代，再来一轮：整理应当只挑这几个
        for k in 5..9 {
            sim.obj_gen[k] = 0;
        }
        let before2 = sim.total_writes;
        sim.sweep = 0;
        step_gen_sep(&mut sim, &[0], 2, 16, 128, 8);
        assert_eq!(sim.total_writes, before2 + 1 + 4, "脏的一个 + 四个热对象，冷的一个都不许进");
    }

    /// **knob 到函数那根线**：`Arm::NodeGroup { g }` 的 g 必须真的传进 `step_node_group`。
    /// ⚠️ 这条是补的：此前 18 条单测**全部直接调** `step_node_group`，
    /// 把函数体里的 `g` 换成常量 8 时 31 个测试一条都不红，而 g2 / g8 / g32 三行会逐字相同、
    /// 判决行随之翻面（2026-09-10 反推腿实测出来的坏法）。
    #[test]
    fn node_group_knob_is_wired_through_run_arm() {
        let a = run_arm(Arm::NodeGroup { g: 2, extra: 128 }, Load::Runs8, 0, 200, 50);
        let b = run_arm(Arm::NodeGroup { g: 8, extra: 128 }, Load::Runs8, 0, 200, 50);
        let c = run_arm(Arm::NodeGroup { g: 32, extra: 128 }, Load::Runs8, 0, 200, 50);
        assert_ne!(a.runs_final, b.runs_final, "g=2 与 g=8 出同一个数 ⇒ knob 没接上");
        assert_ne!(b.runs_final, c.runs_final, "g=8 与 g=32 出同一个数 ⇒ knob 没接上");
        // 绝对值锚点：三个数各自钉死，只钉「互不相等」测不出三条一起错
        assert_eq!((a.runs_final, b.runs_final, c.runs_final), (1862, 897, 1100));
    }

    /// 同一根线，代际分离那一侧：(h1, h2) 必须真的传进 `step_gen_sep`。
    #[test]
    fn gen_sep_knob_is_wired_through_run_arm() {
        let a = run_arm(Arm::GenSep { h1: 16, h2: 128, extra: 128 }, Load::Runs8, 0, 200, 50);
        let b = run_arm(Arm::GenSep { h1: 256, h2: 1024, extra: 128 }, Load::Runs8, 0, 200, 50);
        assert_ne!(a.runs_final, b.runs_final, "两组阈值出同一个数 ⇒ knob 没接上");
        assert_eq!((a.runs_final, b.runs_final), (3442, 2588));
    }

    /// **代际分离的回落必须计数**：没有空段时逐槽回落，counter 要涨。
    #[test]
    fn gen_sep_fallback_counts() {
        let mut sim = Sim::new(8, 16, 4);
        // 段 2、3 是仅有的空段；先把它们的槽全占掉，逼出回落
        for slot in 8u32..16 {
            sim.take_slot(slot);
        }
        // 现在一个空槽都没有 ⇒ 得先还两个不成段的槽回去
        sim.free.insert(9);
        sim.seg_free[9 / 4] += 1;
        sim.free.insert(13);
        sim.seg_free[13 / 4] += 1;
        assert!(sim.open_empty_seg_for(0).is_none(), "不许还有全空段，否则测的不是回落");
        let before = sim.fallback_allocs;
        sim.place_gen_run(0, &[2]);
        assert_eq!(sim.fallback_allocs, before + 1, "回落必须计数");
    }

    /// 代际分离：热与冷用各自的开放段这一点，靠 `gen_open` 分开，
    /// **而 `seg_gen` 那道 tag 检查在本模型里判定为等价变异**——
    /// 一个全空段永远不会带着别人的 tag（end_checkpoint 会清掉，开放段又被 gen_open 排除）。
    /// 这条测试把那个等价性留档（mutation-sampling.md 要求）。
    #[test]
    fn seg_gen_tag_is_never_stale_on_an_empty_segment() {
        let mut sim = Sim::new(8, 16, 4);
        sim.place_gen_run(0, &[0, 1]);
        let seg = sim.slot_of[0] as usize / sim.g;
        assert_eq!(sim.seg_gen[seg], Some(0));
        // 把这个段腾空，走一次 checkpoint 收尾
        sim.place_gen_run(2, &[0, 1]);
        sim.gen_open[0] = None;
        sim.end_checkpoint();
        assert_eq!(sim.seg_free[seg] as usize, sim.g, "段已回空");
        assert_eq!(sim.seg_gen[seg], None, "回空的段必须丢掉代身份");
    }

    /// 老化代分档是纯算术，边界钉死（三档、两个阈值、左闭右开）。
    #[test]
    fn age_gen_boundaries() {
        assert_eq!(age_gen(0, 16, 128), 0);
        assert_eq!(age_gen(15, 16, 128), 0);
        assert_eq!(age_gen(16, 16, 128), 1);
        assert_eq!(age_gen(127, 16, 128), 1);
        assert_eq!(age_gen(128, 16, 128), 2);
        assert_eq!(age_gen(1_000_000, 16, 128), 2);
    }

    /// 预算封顶的邻居扩展：花的钱不许超过预算，且预算够大时必须真花出去。
    #[test]
    fn budgeted_neighbors_respect_budget() {
        let dirty: Vec<usize> = vec![100, 200, 300];
        let out = extend_neighbors_budgeted(&dirty, 6, 8192);
        assert_eq!(out.len(), 9, "3 个脏 + 6 个邻居");
        let none = extend_neighbors_budgeted(&dirty, 0, 8192);
        assert_eq!(none.len(), 3, "预算 0 就一个邻居都不许扩");
        let capped = extend_neighbors_budgeted(&dirty, 1, 8192);
        assert_eq!(capped.len(), 4, "预算 1 只许扩一个");
    }

    /// 代际分离的分区断言：热与冷永不落进同一个段。
    #[test]
    fn gen_segments_never_mix() {
        let mut sim = Sim::new(64, 128, 8);
        sim.place_gen_run(0, &[0, 1, 2]);
        sim.place_gen_run(2, &[40, 41]);
        let hot_seg = sim.slot_of[0] as usize / sim.g;
        let cold_seg = sim.slot_of[40] as usize / sim.g;
        assert_ne!(hot_seg, cold_seg, "热与冷落进了同一个段");
        assert_eq!(sim.seg_gen[hot_seg], Some(0));
        assert_eq!(sim.seg_gen[cold_seg], Some(2));
    }

    /// `find_free_run` 是 place_chunk 的第三条分支，手算钉死它找哪一段。
    #[test]
    fn find_free_run_picks_lowest_contiguous() {
        let mut sim = Sim::new(8, 16, 4);
        // 初始：key 0..7 住槽 0..7，空槽 8..15（段 2 与段 3 全空）
        assert_eq!(sim.find_free_run(4), Some(8));
        // 把 12 与 13 占掉 ⇒ 长 4 的 run 只剩不到，长 2 的还有
        sim.take_slot(12);
        sim.take_slot(13);
        assert_eq!(sim.find_free_run(4), Some(8));
        sim.take_slot(9);
        assert_eq!(sim.find_free_run(4), None, "剩下的空槽拼不出长 4 的 run");
        assert_eq!(sim.find_free_run(2), Some(10));
    }

    /// place_chunk 的第四条分支（逐槽回落）必须能被强制进入，且计数。
    #[test]
    fn place_chunk_falls_back_and_counts() {
        let mut sim = Sim::new(8, 16, 4);
        // 占掉除 9、11、13、15 之外的空槽 ⇒ 没有任何长 2 的连续 run，也没有全空段
        for slot in [8u32, 10, 12, 14] {
            sim.take_slot(slot);
        }
        assert_eq!(sim.find_free_run(2), None);
        assert_eq!(sim.open_empty_seg(), None);
        let before = sim.fallback_allocs;
        sim.place_run(&[2, 3]);
        assert_eq!(sim.fallback_allocs, before + 2, "两个 key 各回落一次");
    }

    /// 节点组臂的绝对值：组大小 g、预算够时，一个 checkpoint 写的对象数
    /// 恰是「被碰到的组的并集」的大小，不是脏对象数。
    #[test]
    fn node_group_writes_whole_groups() {
        let mut sim = Sim::new(64, 128, 8);
        let g = 4usize;
        let dirty = [5usize, 6, 20];
        // 组 1 = {4,5,6,7}，组 5 = {20,21,22,23} ⇒ 并集 8 个对象
        let dirty_set_: BTreeSet<usize> = dirty.iter().copied().collect();
        let mut touched: Vec<usize> = dirty.iter().map(|&k| k / g).collect();
        touched.dedup();
        assert_eq!(touched, vec![1, 5]);
        for grp in touched {
            let keys: Vec<usize> = (grp * g..(grp + 1) * g).collect();
            let clean = keys.iter().filter(|k| !dirty_set_.contains(k)).count();
            assert_eq!(clean, if grp == 1 { 2 } else { 3 });
            sim.place_run(&keys);
        }
        assert_eq!(sim.total_writes, 8, "两个组各 4 个对象");
    }
}
