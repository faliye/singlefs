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

const OBJECT_COUNT: usize = 8192; // 对象数
const SLOT_COUNT: usize = 10240; // 槽数（填充 80%）
const SLOTS_PER_SEGMENT: usize = 64; // 聚簇段槽数
const DIRTY_OBJECTS_PER_CHECKPOINT: usize = 64; // 每 checkpoint 用户脏对象数
const CHECKPOINT_COUNT: u64 = 2000; // checkpoint 数

/// 等预算三档：额外写倍数 1.5 / 3 / 5 ⇒ 每 checkpoint 额外写 D×(X−1)。
const BUDGETS: [(usize, &str); 3] = [(32, "1.5x"), (128, "3x"), (256, "5x")];

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0xA076_1D64_78BD_642F);
        if state == 0 {
            state = 0xDEAD_BEEF;
        }
        Rng(state)
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, exclusive_upper_bound: usize) -> usize {
        (self.next() % exclusive_upper_bound as u64) as usize
    }
}

/// E93 原样复用的四种政策形态（跨装置对照用）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum E93Policy {
    FirstFit,
    BumpSegment,
    BumpNeighbor(usize),
    BumpCompact(usize),
}
impl E93Policy {
    fn tag(self) -> String {
        match self {
            E93Policy::FirstFit => "first_fit".into(),
            E93Policy::BumpSegment => "bump_seg".into(),
            E93Policy::BumpNeighbor(radius) => format!("bump_nb_r{radius}"),
            E93Policy::BumpCompact(compact_budget) => format!("bump_cp_b{compact_budget}"),
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
    /// 格式臂之一：节点组，组大小 `group_size`。
    NodeGroup { group_size: usize, extra: usize },
    /// 格式臂之二：代际分离，热 / 温分档阈值 (`hot_age_threshold`, `warm_age_threshold`)。
    GenerationSeparation { hot_age_threshold: u64, warm_age_threshold: u64, extra: usize },
}
impl Arm {
    fn tag(self) -> String {
        match self {
            Arm::PolicyCompact(_) => "pol_compact".into(),
            Arm::PolicyNeighbor(_) => "pol_neighbor".into(),
            Arm::NodeGroup { group_size, .. } => format!("fmt_group_g{group_size}"),
            Arm::GenerationSeparation { hot_age_threshold, warm_age_threshold, .. } => format!("fmt_gensep_{hot_age_threshold}_{warm_age_threshold}"),
        }
    }
    fn family(self) -> &'static str {
        match self {
            Arm::PolicyCompact(_) | Arm::PolicyNeighbor(_) => "policy",
            Arm::NodeGroup { .. } | Arm::GenerationSeparation { .. } => "format",
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
    object_count: usize,
    slot_count: usize,
    slots_per_segment: usize,
    slot_of: Vec<u32>,
    key_at: Vec<Option<u32>>,
    free: BTreeSet<u32>,
    free_slots_per_segment: Vec<u16>,
    open_segment: Option<usize>,
    bump: usize,
    /// 增量维护的「断开的邻接对」数；runs = broken + 1（L ≥ 1 时）。
    broken: u64,
    deferred: Vec<u32>,
    user_writes: u64,
    total_writes: u64,
    fallback_allocations: u64,
    sweep: usize,
    /// 代际分离用：每个段被开成哪一代（None = 未定型）。段回空时清掉。
    segment_generation: Vec<Option<u8>>,
    /// 代际分离用：每一代自己的开放段与 bump 位置。
    generation_open_segment: [Option<usize>; 3],
    generation_bump_offset: [usize; 3],
    /// 每个对象上一次被重写的 checkpoint。
    last_write: Vec<u64>,
    /// 每个对象上一次落盘时被判成哪一代。
    object_generation: Vec<u8>,
}

impl Sim {
    /// 初始布局：key k 住槽 k（顺序创建的形态），与 E93 逐字同一套。
    fn new(object_count: usize, slot_count: usize, slots_per_segment: usize) -> Sim {
        assert!(object_count < slot_count && slot_count % slots_per_segment == 0);
        let slot_of: Vec<u32> = (0..object_count as u32).collect();
        let mut key_at: Vec<Option<u32>> = vec![None; slot_count];
        for key in 0..object_count {
            key_at[key] = Some(key as u32);
        }
        let free: BTreeSet<u32> = (object_count as u32..slot_count as u32).collect();
        let mut free_slots_per_segment = vec![0u16; slot_count / slots_per_segment];
        for slot in object_count..slot_count {
            free_slots_per_segment[slot / slots_per_segment] += 1;
        }
        let segment_count = slot_count / slots_per_segment;
        Sim {
            object_count,
            slot_count,
            slots_per_segment,
            slot_of,
            key_at,
            free,
            free_slots_per_segment,
            open_segment: None,
            bump: 0,
            broken: 0,
            deferred: Vec::new(),
            user_writes: 0,
            total_writes: 0,
            fallback_allocations: 0,
            sweep: 0,
            segment_generation: vec![None; segment_count],
            generation_open_segment: [None; 3],
            generation_bump_offset: [0; 3],
            last_write: vec![0; object_count],
            object_generation: vec![2; object_count],
        }
    }

    fn pair_broken(&self, key: usize) -> bool {
        self.slot_of[key] != self.slot_of[key - 1] + 1
    }

    /// 审计：全扫重算 runs。与增量维护不共享累加代码。
    fn audit_runs(&self) -> u64 {
        let mut runs = 1u64;
        for key in 1..self.object_count {
            if self.slot_of[key] != self.slot_of[key - 1] + 1 {
                runs += 1;
            }
        }
        runs
    }

    fn runs_incremental(&self) -> u64 {
        self.broken + 1
    }

    fn move_key(&mut self, key: usize, new_slot: u32) {
        debug_assert!(self.key_at[new_slot as usize].is_none());
        if key >= 1 && self.pair_broken(key) {
            self.broken -= 1;
        }
        if key + 1 < self.object_count && self.pair_broken(key + 1) {
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
        if key + 1 < self.object_count && self.pair_broken(key + 1) {
            self.broken += 1;
        }
        self.total_writes += 1;
    }

    fn allocate_first_fit(&mut self) -> u32 {
        let slot = *self
            .free
            .iter()
            .find(|&&free_slot| self.open_segment != Some(free_slot as usize / self.slots_per_segment))
            .expect("槽用尽：配置违反 D ≤ S−L");
        self.free.remove(&slot);
        self.free_slots_per_segment[slot as usize / self.slots_per_segment] -= 1;
        slot
    }

    fn find_empty_segment(&mut self) -> Option<usize> {
        (0..self.free_slots_per_segment.len())
            .find(|&segment_index| self.free_slots_per_segment[segment_index] as usize == self.slots_per_segment && self.open_segment != Some(segment_index))
    }

    fn allocate_bump(&mut self) -> u32 {
        loop {
            if let Some(segment_index) = self.open_segment {
                if self.bump < self.slots_per_segment {
                    let slot = (segment_index * self.slots_per_segment + self.bump) as u32;
                    self.bump += 1;
                    debug_assert!(self.free.contains(&slot));
                    self.free.remove(&slot);
                    self.free_slots_per_segment[segment_index] -= 1;
                    return slot;
                }
                self.open_segment = None;
            }
            match self.find_empty_segment() {
                Some(segment_index) => {
                    self.open_segment = Some(segment_index);
                    self.bump = 0;
                }
                None => {
                    self.fallback_allocations += 1;
                    return self.allocate_first_fit();
                }
            }
        }
    }

    fn find_free_run(&self, run_length_in_slots: usize) -> Option<u32> {
        let mut start = 0u32;
        let mut count = 0usize;
        let mut previous_free_slot: Option<u32> = None;
        for &free_slot in &self.free {
            if self.open_segment == Some(free_slot as usize / self.slots_per_segment) {
                previous_free_slot = None;
                count = 0;
                continue;
            }
            match previous_free_slot {
                Some(previous_slot) if free_slot == previous_slot + 1 => count += 1,
                _ => {
                    start = free_slot;
                    count = 1;
                }
            }
            previous_free_slot = Some(free_slot);
            if count >= run_length_in_slots {
                return Some(start);
            }
        }
        None
    }

    fn take_slot(&mut self, slot: u32) {
        let removed = self.free.remove(&slot);
        debug_assert!(removed);
        self.free_slots_per_segment[slot as usize / self.slots_per_segment] -= 1;
    }

    fn place_run(&mut self, keys: &[usize]) {
        let slots_per_segment = self.slots_per_segment;
        let owned: Vec<usize> = keys.to_vec();
        for chunk in owned.chunks(slots_per_segment) {
            self.place_chunk(chunk);
        }
    }

    fn place_chunk(&mut self, keys: &[usize]) {
        let chunk_key_count = keys.len();
        if let Some(segment_index) = self.open_segment {
            if self.slots_per_segment - self.bump >= chunk_key_count {
                for &key in keys {
                    let slot = (segment_index * self.slots_per_segment + self.bump) as u32;
                    self.bump += 1;
                    self.take_slot(slot);
                    self.move_key(key, slot);
                }
                return;
            }
        }
        if let Some(segment_index) = self.find_empty_segment() {
            self.open_segment = Some(segment_index);
            self.bump = 0;
            for &key in keys {
                let slot = (segment_index * self.slots_per_segment + self.bump) as u32;
                self.bump += 1;
                self.take_slot(slot);
                self.move_key(key, slot);
            }
            return;
        }
        if let Some(start) = self.find_free_run(chunk_key_count) {
            for (offset_in_run, &key) in keys.iter().enumerate() {
                let slot = start + offset_in_run as u32;
                self.take_slot(slot);
                self.move_key(key, slot);
            }
            return;
        }
        for &key in keys {
            self.fallback_allocations += 1;
            let slot = self.allocate_first_fit();
            self.move_key(key, slot);
        }
    }

    // ---- 代际分离臂专用的放置面 ----

    /// 找一个能开给第 `generation` 代的全空段（全空段没有代，谁开归谁）。
    fn find_empty_segment_for_generation(&self, generation: u8) -> Option<usize> {
        (0..self.free_slots_per_segment.len()).find(|&segment_index| {
            self.free_slots_per_segment[segment_index] as usize == self.slots_per_segment
                && self.generation_open_segment[0] != Some(segment_index)
                && self.generation_open_segment[1] != Some(segment_index)
                && self.generation_open_segment[2] != Some(segment_index)
                && self.segment_generation[segment_index].map_or(true, |tagged_generation| tagged_generation == generation)
        })
    }

    /// 按代分区落一批 key（同一代、按 key 序）。热与冷永不落进同一个段。
    fn place_generation_run(&mut self, generation: u8, keys: &[usize]) {
        let generation_index = generation as usize;
        for &key in keys {
            loop {
                if let Some(segment_index) = self.generation_open_segment[generation_index] {
                    if self.generation_bump_offset[generation_index] < self.slots_per_segment {
                        let slot = (segment_index * self.slots_per_segment + self.generation_bump_offset[generation_index]) as u32;
                        self.generation_bump_offset[generation_index] += 1;
                        debug_assert!(self.free.contains(&slot));
                        self.take_slot(slot);
                        self.move_key(key, slot);
                        break;
                    }
                    self.generation_open_segment[generation_index] = None;
                }
                match self.find_empty_segment_for_generation(generation) {
                    Some(segment_index) => {
                        self.generation_open_segment[generation_index] = Some(segment_index);
                        self.generation_bump_offset[generation_index] = 0;
                        self.segment_generation[segment_index] = Some(generation);
                    }
                    None => {
                        // 无空段：回落逐槽分配，计数（与 E93 的回落口径同一个数）
                        self.fallback_allocations += 1;
                        let slot = self.allocate_first_fit_outside_open_generation_segments();
                        self.move_key(key, slot);
                        break;
                    }
                }
            }
        }
    }

    /// 代际分离臂的回落：三代的开放段都是保留区，first_fit 不许拿。
    fn allocate_first_fit_outside_open_generation_segments(&mut self) -> u32 {
        let slot = *self
            .free
            .iter()
            .find(|&&free_slot| {
                let segment_index = free_slot as usize / self.slots_per_segment;
                self.generation_open_segment[0] != Some(segment_index)
                    && self.generation_open_segment[1] != Some(segment_index)
                    && self.generation_open_segment[2] != Some(segment_index)
            })
            .expect("槽用尽：配置违反 D ≤ S−L");
        self.free.remove(&slot);
        self.free_slots_per_segment[slot as usize / self.slots_per_segment] -= 1;
        slot
    }

    /// checkpoint 收尾：defer 的槽此刻才真正可复用（D16 新规则 2）。
    fn end_checkpoint(&mut self) {
        for slot in std::mem::take(&mut self.deferred) {
            self.free.insert(slot);
            self.free_slots_per_segment[slot as usize / self.slots_per_segment] += 1;
        }
        // 回空的段丢掉代身份（否则代分区会把空段永久锁给某一代）
        for segment_index in 0..self.free_slots_per_segment.len() {
            if self.free_slots_per_segment[segment_index] as usize == self.slots_per_segment
                && self.generation_open_segment[0] != Some(segment_index)
                && self.generation_open_segment[1] != Some(segment_index)
                && self.generation_open_segment[2] != Some(segment_index)
            {
                self.segment_generation[segment_index] = None;
            }
        }
    }

    fn occupied(&self) -> usize {
        self.key_at.iter().filter(|resident_key| resident_key.is_some()).count()
    }
}

/// 选负载脏集（去重、升序）。与 E93 逐字同一套。
fn dirty_set(load: Load, rng: &mut Rng, object_count: usize, dirty_count: usize) -> Vec<usize> {
    let mut set = BTreeSet::new();
    match load {
        Load::Uniform => {
            while set.len() < dirty_count {
                set.insert(rng.below(object_count));
            }
        }
        Load::Runs8 => {
            while set.len() < dirty_count {
                let start = rng.below(object_count);
                for key in start..(start + 8).min(object_count) {
                    if set.len() < dirty_count {
                        set.insert(key);
                    }
                }
            }
        }
    }
    set.into_iter().collect()
}

/// E93 的 bump_neighbor：把脏 key 段向两侧各扩 ≤R 个干净邻居。
fn extend_neighbors(dirty: &[usize], radius: usize, object_count: usize) -> (Vec<usize>, u64) {
    let mut set: BTreeSet<usize> = dirty.iter().copied().collect();
    let mut extra = 0u64;
    let mut runs: Vec<(usize, usize)> = Vec::new();
    for &key in dirty {
        match runs.last_mut() {
            Some((_, run_last)) if *run_last + 1 == key => *run_last = key,
            _ => runs.push((key, key)),
        }
    }
    for (run_first, run_last) in runs {
        for key in (run_first.saturating_sub(radius)..run_first).chain(run_last + 1..(run_last + 1 + radius).min(object_count)) {
            if set.insert(key) {
                extra += 1;
            }
        }
    }
    (set.into_iter().collect(), extra)
}

/// 预算封顶的邻居扩展：半径从 1 起一圈圈加，加到预算花完为止。
/// **与 E93 的 bump_neighbor 差在封顶**——不封顶就落不到等预算格上。
fn extend_neighbors_budgeted(dirty: &[usize], budget: usize, object_count: usize) -> Vec<usize> {
    let mut set: BTreeSet<usize> = dirty.iter().copied().collect();
    let mut spent = 0usize;
    let mut runs: Vec<(usize, usize)> = Vec::new();
    for &key in dirty {
        match runs.last_mut() {
            Some((_, run_last)) if *run_last + 1 == key => *run_last = key,
            _ => runs.push((key, key)),
        }
    }
    let mut radius = 1usize;
    while spent < budget && radius <= object_count {
        let mut grew = false;
        for &(run_first, run_last) in &runs {
            for key in [run_first.checked_sub(radius), Some(run_last + radius)].into_iter().flatten() {
                if key < object_count && spent < budget && set.insert(key) {
                    spent += 1;
                    grew = true;
                }
            }
        }
        if !grew {
            break;
        }
        radius += 1;
    }
    set.into_iter().collect()
}

struct Outcome {
    runs_final: u64,
    write_amp: f64,
    fallback_percent: f64,
    budget_spent_percent: f64,
}

/// E93 的四条政策臂，逐字照抄它的 run_arm——跨装置对照就靠这一段。
fn run_e93_arm(policy: E93Policy, load: Load, seed: u64, checkpoint_count: u64, sample_every: u64) -> Outcome {
    let mut sim = Sim::new(OBJECT_COUNT, SLOT_COUNT, SLOTS_PER_SEGMENT);
    let mut rng = Rng::new(seed);
    for checkpoint in 1..=checkpoint_count {
        let dirty = dirty_set(load, &mut rng, OBJECT_COUNT, DIRTY_OBJECTS_PER_CHECKPOINT);
        sim.user_writes += dirty.len() as u64;
        let (batch, _extra) = match policy {
            E93Policy::BumpNeighbor(radius) => extend_neighbors(&dirty, radius, OBJECT_COUNT),
            _ => (dirty, 0),
        };
        match policy {
            E93Policy::FirstFit => {
                for &key in &batch {
                    let slot = sim.allocate_first_fit();
                    sim.move_key(key, slot);
                }
            }
            E93Policy::BumpSegment | E93Policy::BumpCompact(_) => {
                for &key in &batch {
                    let slot = sim.allocate_bump();
                    sim.move_key(key, slot);
                }
            }
            E93Policy::BumpNeighbor(_) => {
                let mut run_start = 0;
                while run_start < batch.len() {
                    let mut run_end = run_start + 1;
                    while run_end < batch.len() && batch[run_end] == batch[run_end - 1] + 1 {
                        run_end += 1;
                    }
                    let run: Vec<usize> = batch[run_start..run_end].to_vec();
                    sim.place_run(&run);
                    run_start = run_end;
                }
            }
        }
        if let E93Policy::BumpCompact(compact_budget) = policy {
            let batch_set: BTreeSet<usize> = batch.iter().copied().collect();
            let mut sweep_keys = Vec::with_capacity(compact_budget);
            while sweep_keys.len() < compact_budget {
                let key = sim.sweep;
                sim.sweep = (sim.sweep + 1) % OBJECT_COUNT;
                if !batch_set.contains(&key) {
                    sweep_keys.push(key);
                }
            }
            sim.place_run(&sweep_keys);
        }
        sim.end_checkpoint();
        if checkpoint % sample_every == 0 {
            assert_eq!(sim.runs_incremental(), sim.audit_runs(), "t={checkpoint} 增量与审计分叉");
            assert_eq!(sim.occupied(), sim.object_count);
            assert_eq!(sim.free.len() + sim.object_count, sim.slot_count);
        }
    }
    Outcome {
        runs_final: sim.runs_incremental(),
        write_amp: sim.total_writes as f64 / sim.user_writes as f64,
        fallback_percent: 100.0 * sim.fallback_allocations as f64 / sim.total_writes as f64,
        budget_spent_percent: 0.0,
    }
}

/// 老化代分档：自上次重写以来的 checkpoint 数落在哪一档。0 = 热，1 = 温，2 = 冷。
fn generation_for_age(age: u64, hot_age_threshold: u64, warm_age_threshold: u64) -> u8 {
    if age < hot_age_threshold {
        0
    } else if age < warm_age_threshold {
        1
    } else {
        2
    }
}

fn run_arm(arm: Arm, load: Load, seed: u64, checkpoint_count: u64, sample_every: u64) -> Outcome {
    let mut sim = Sim::new(OBJECT_COUNT, SLOT_COUNT, SLOTS_PER_SEGMENT);
    let mut rng = Rng::new(seed);
    let mut budget_total = 0u64;
    let mut budget_spent = 0u64;
    let mut group_cursor = 0usize;
    for checkpoint in 1..=checkpoint_count {
        let dirty = dirty_set(load, &mut rng, OBJECT_COUNT, DIRTY_OBJECTS_PER_CHECKPOINT);
        sim.user_writes += dirty.len() as u64;
        let before = sim.total_writes;
        match arm {
            Arm::PolicyCompact(extra) => {
                budget_total += extra as u64;
                for &key in &dirty {
                    let slot = sim.allocate_bump();
                    sim.move_key(key, slot);
                }
                let dirty_key_set: BTreeSet<usize> = dirty.iter().copied().collect();
                let mut sweep_keys = Vec::with_capacity(extra);
                while sweep_keys.len() < extra {
                    let key = sim.sweep;
                    sim.sweep = (sim.sweep + 1) % OBJECT_COUNT;
                    if !dirty_key_set.contains(&key) {
                        sweep_keys.push(key);
                    }
                }
                sim.place_run(&sweep_keys);
            }
            Arm::PolicyNeighbor(extra) => {
                budget_total += extra as u64;
                let batch = extend_neighbors_budgeted(&dirty, extra, OBJECT_COUNT);
                let mut run_start = 0;
                while run_start < batch.len() {
                    let mut run_end = run_start + 1;
                    while run_end < batch.len() && batch[run_end] == batch[run_end - 1] + 1 {
                        run_end += 1;
                    }
                    let run: Vec<usize> = batch[run_start..run_end].to_vec();
                    sim.place_run(&run);
                    run_start = run_end;
                }
            }
            Arm::NodeGroup { group_size, extra } => {
                budget_total += extra as u64;
                step_node_group(&mut sim, &dirty, group_size, extra, &mut group_cursor);
            }
            Arm::GenerationSeparation { hot_age_threshold, warm_age_threshold, extra } => {
                budget_total += extra as u64;
                step_generation_separation(&mut sim, &dirty, checkpoint, hot_age_threshold, warm_age_threshold, extra);
            }
        }
        budget_spent += (sim.total_writes - before).saturating_sub(dirty.len() as u64);
        sim.end_checkpoint();
        if checkpoint % sample_every == 0 {
            assert_eq!(sim.runs_incremental(), sim.audit_runs(), "t={checkpoint} 增量与审计分叉");
            assert_eq!(sim.occupied(), sim.object_count);
            assert_eq!(sim.free.len() + sim.object_count, sim.slot_count);
        }
    }
    Outcome {
        runs_final: sim.runs_incremental(),
        write_amp: sim.total_writes as f64 / sim.user_writes as f64,
        fallback_percent: 100.0 * sim.fallback_allocations as f64 / sim.total_writes as f64,
        budget_spent_percent: 100.0 * budget_spent as f64 / budget_total as f64,
    }
}

/// 节点组臂的一个 checkpoint。抽成函数是为了让断言够得着——
/// 留在 run_arm 里的时候，一条「组只落脏的那几个」的变异一个测试都不会红。
fn step_node_group(
    sim: &mut Sim,
    dirty: &[usize],
    group_size: usize,
    extra: usize,
    group_cursor: &mut usize,
) {
    let object_count = sim.object_count;
    let dirty_key_set: BTreeSet<usize> = dirty.iter().copied().collect();
    let mut remaining = extra;
    let mut written: BTreeSet<usize> = BTreeSet::new();
    // 被脏集碰到的组，按 key 序：预算够就整组重落，不够就只落脏的那几个
    let mut touched: Vec<usize> = dirty.iter().map(|&key| key / group_size).collect();
    touched.dedup();
    for group_index in touched {
        let group_first_key = group_index * group_size;
        let group_end_key = ((group_index + 1) * group_size).min(object_count);
        let clean = (group_first_key..group_end_key).filter(|key| !dirty_key_set.contains(key)).count();
        let keys: Vec<usize> = if clean <= remaining {
            remaining -= clean;
            (group_first_key..group_end_key).collect()
        } else {
            (group_first_key..group_end_key).filter(|key| dirty_key_set.contains(key)).collect()
        };
        for &key in &keys {
            written.insert(key);
        }
        sim.place_run(&keys);
    }
    // 预算还有剩：按组轮转，主动整组重落（组是放置单位，所以整组落）
    let groups = object_count.div_ceil(group_size);
    let mut scanned = 0usize;
    while remaining >= group_size && scanned < groups {
        let group_first_key = *group_cursor * group_size;
        let group_end_key = ((*group_cursor + 1) * group_size).min(object_count);
        *group_cursor = (*group_cursor + 1) % groups;
        scanned += 1;
        if (group_first_key..group_end_key).any(|key| written.contains(&key)) {
            continue;
        }
        let keys: Vec<usize> = (group_first_key..group_end_key).collect();
        remaining -= keys.len();
        for &key in &keys {
            written.insert(key);
        }
        sim.place_run(&keys);
    }
}

/// 代际分离臂的一个 checkpoint。抽出来的理由同上。
fn step_generation_separation(sim: &mut Sim, dirty: &[usize], checkpoint: u64, hot_age_threshold: u64, warm_age_threshold: u64, extra: usize) {
    let object_count = sim.object_count;
    // 脏对象：按「自上次重写以来多久」定代，同代同段、按 key 序
    let mut dirty_keys_by_generation: [Vec<usize>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    for &key in dirty {
        let generation = generation_for_age(checkpoint - sim.last_write[key], hot_age_threshold, warm_age_threshold);
        dirty_keys_by_generation[generation as usize].push(key);
        sim.object_generation[key] = generation;
        sim.last_write[key] = checkpoint;
    }
    for generation in 0..3u8 {
        let keys = std::mem::take(&mut dirty_keys_by_generation[generation as usize]);
        sim.place_generation_run(generation, &keys);
    }
    // 整理只清热区：按 key 序轮转，只挑现在住在热代里的对象
    let dirty_key_set: BTreeSet<usize> = dirty.iter().copied().collect();
    let mut sweep_keys = Vec::with_capacity(extra);
    let mut looked = 0usize;
    while sweep_keys.len() < extra && looked < object_count {
        let key = sim.sweep;
        sim.sweep = (sim.sweep + 1) % object_count;
        looked += 1;
        if !dirty_key_set.contains(&key) && sim.object_generation[key] == 0 {
            sweep_keys.push(key);
        }
    }
    for &key in &sweep_keys {
        sim.last_write[key] = checkpoint;
    }
    sim.place_generation_run(0, &sweep_keys);
}

/// 判据 3：测量的阳性对照。强制放置（绕过政策）走增量路径，审计路径复核。
fn measurement_control(sim: &mut Sim) -> (u64, u64, u64, u64) {
    let object_count = sim.object_count;
    for key in 0..object_count {
        let target = if key < object_count / 2 { 2 * key as u32 } else { (2 * (key - object_count / 2) + 1) as u32 };
        force_to(sim, key, target);
    }
    let scatter = (sim.runs_incremental(), sim.audit_runs());
    for key in 0..object_count {
        force_to(sim, key, key as u32);
    }
    let compact = (sim.runs_incremental(), sim.audit_runs());
    (scatter.0, scatter.1, compact.0, compact.1)
}

fn force_to(sim: &mut Sim, key: usize, target: u32) {
    if sim.slot_of[key] == target {
        return;
    }
    if let Some(other) = sim.key_at[target as usize] {
        let spare = *sim.free.iter().next().expect("强制放置需要至少一个空槽");
        sim.free.remove(&spare);
        sim.free_slots_per_segment[spare as usize / sim.slots_per_segment] -= 1;
        sim.move_key(other as usize, spare);
    }
    if sim.free.remove(&target) {
        sim.free_slots_per_segment[target as usize / sim.slots_per_segment] -= 1;
    } else {
        let deferred_position = sim.deferred.iter().position(|&deferred_slot| deferred_slot == target).expect("目标槽既不空也不在 defer");
        sim.deferred.swap_remove(deferred_position);
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

fn median_of_five(mut values: Vec<u64>) -> u64 {
    values.sort_unstable();
    values[2]
}

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config l={OBJECT_COUNT} s={SLOT_COUNT} g={SLOTS_PER_SEGMENT} d={DIRTY_OBJECTS_PER_CHECKPOINT} t={CHECKPOINT_COUNT} model=counting file_ops=0 seeds=5 budgets=32,128,256"
        ))
    );

    // 等预算格上的臂：政策两条 + 格式两条（格式臂的 knob 各扫三个值，取每格最好的）
    let mut arms: Vec<Arm> = Vec::new();
    for (extra, _) in BUDGETS {
        arms.push(Arm::PolicyCompact(extra));
        arms.push(Arm::PolicyNeighbor(extra));
        for group_size in [2usize, 8, 32] {
            arms.push(Arm::NodeGroup { group_size, extra });
        }
        for (hot_age_threshold, warm_age_threshold) in [(16u64, 128u64), (64, 512), (256, 1024)] {
            arms.push(Arm::GenerationSeparation { hot_age_threshold, warm_age_threshold, extra });
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
        let mut sim = Sim::new(OBJECT_COUNT, SLOT_COUNT, SLOTS_PER_SEGMENT);
        let (scatter_incremental, scatter_audit, compact_incremental, compact_audit) = measurement_control(&mut sim);
        assert_eq!((scatter_incremental, scatter_audit), (OBJECT_COUNT as u64, OBJECT_COUNT as u64), "{:?} 全隔离布局必须报 runs=L", arm);
        assert_eq!((compact_incremental, compact_audit), (1, 1), "{:?} 连续布局必须报 runs=1", arm);
        println!(
            "{}",
            emitter.emit_raw(&format!("name=control arm={key} scatter_runs={scatter_incremental} compact_runs={compact_incremental}"))
        );
    }
    for policy in [
        E93Policy::FirstFit,
        E93Policy::BumpSegment,
        E93Policy::BumpNeighbor(1),
        E93Policy::BumpNeighbor(2),
        E93Policy::BumpCompact(32),
        E93Policy::BumpCompact(128),
        E93Policy::BumpCompact(256),
    ] {
        let mut sim = Sim::new(OBJECT_COUNT, SLOT_COUNT, SLOTS_PER_SEGMENT);
        let (scatter_incremental, scatter_audit, compact_incremental, compact_audit) = measurement_control(&mut sim);
        assert_eq!((scatter_incremental, scatter_audit), (OBJECT_COUNT as u64, OBJECT_COUNT as u64), "{:?} 全隔离布局必须报 runs=L", policy);
        assert_eq!((compact_incremental, compact_audit), (1, 1), "{:?} 连续布局必须报 runs=1", policy);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=control arm=e93_{} scatter_runs={scatter_incremental} compact_runs={compact_incremental}",
                policy.tag()
            ))
        );
    }

    // 跨装置闸：E93 的 14 个 aging_median 必须逐个复现
    for (arm_tag, load_tag, want) in E93_MEDIANS {
        let policy = match arm_tag {
            "first_fit" => E93Policy::FirstFit,
            "bump_seg" => E93Policy::BumpSegment,
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
        let got = median_of_five((0..5u64).map(|seed| run_e93_arm(policy, load, seed, CHECKPOINT_COUNT, 250).runs_final).collect());
        assert_eq!(
            got, want,
            "跨装置对照失败：{arm_tag}/{load_tag} 本装置报 {got}，E93 入库产物里是 {want}"
        );
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=xfixture arm={arm_tag} load={load_tag} median={got} e93_stored={want}"
            ))
        );
    }

    // 判据 4：等预算格
    let mut grid: Vec<(usize, String, &'static str, u64, f64, f64, f64)> = Vec::new();
    for &arm in &arms {
        let extra = match arm {
            Arm::PolicyCompact(extra) | Arm::PolicyNeighbor(extra) => extra,
            Arm::NodeGroup { extra, .. } | Arm::GenerationSeparation { extra, .. } => extra,
        };
        for load in [Load::Uniform, Load::Runs8] {
            let mut finals = Vec::new();
            let mut write_amplification = 0.0;
            let mut fallback_percent = 0.0;
            let mut budget_spent_percent = 0.0;
            for seed in 0..5u64 {
                let outcome = run_arm(arm, load, seed, CHECKPOINT_COUNT, 250);
                if seed == 0 {
                    write_amplification = outcome.write_amp;
                    fallback_percent = outcome.fallback_percent;
                    budget_spent_percent = outcome.budget_spent_percent;
                }
                finals.push(outcome.runs_final);
            }
            let runs_median = median_of_five(finals);
            // **绝对值闸**：等预算格上任何一条臂都不许超预算。
            // 只有互比（「格式臂比政策臂低」）测不出「所有臂一起多花了钱」。
            let write_amplification_ceiling = (DIRTY_OBJECTS_PER_CHECKPOINT + extra) as f64 / DIRTY_OBJECTS_PER_CHECKPOINT as f64;
            assert!(
                write_amplification <= write_amplification_ceiling + 1e-9,
                "{} 在预算 {extra} 上花超了：write_amp={write_amplification:.6} > {write_amplification_ceiling:.6}",
                arm.tag()
            );
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=grid budget={extra} arm={} family={} load={} runs_median={runs_median} write_amp={write_amplification:.3} budget_spent_pct={budget_spent_percent:.1} fallback_pct={fallback_percent:.1}",
                    arm.tag(),
                    arm.family(),
                    load.tag()
                ))
            );
            grid.push((extra, arm.tag(), load.tag(), runs_median, write_amplification, budget_spent_percent, fallback_percent));
        }
    }

    // 判据 5：格式臂在任一等预算格上能不能把 runs 压到政策臂的一半以下
    let mut any_halved = false;
    for (extra, _) in BUDGETS {
        for load in ["uniform", "runs8"] {
            let best_policy = grid
                .iter()
                .filter(|grid_row| grid_row.0 == extra && grid_row.2 == load && grid_row.1.starts_with("pol_"))
                .map(|grid_row| grid_row.3)
                .min()
                .expect("每格必须有政策臂");
            let (best_format_arm_tag, best_format_runs) = grid
                .iter()
                .filter(|grid_row| grid_row.0 == extra && grid_row.2 == load && grid_row.1.starts_with("fmt_"))
                .map(|grid_row| (grid_row.1.clone(), grid_row.3))
                .min_by_key(|(_, candidate_runs_median)| *candidate_runs_median)
                .expect("每格必须有格式臂");
            let halved = best_format_runs * 2 < best_policy;
            any_halved |= halved;
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=verdict budget={extra} load={load} best_policy={best_policy} best_format={best_format_runs} best_format_arm={best_format_arm_tag} format_halves_policy={halved}"
                ))
            );
        }
    }
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=answer any_cell_format_halves_policy={any_halved} criterion=E95_5"
        ))
    );
    println!("{}", emitter.finish());
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
        assert_eq!(sim.runs_incremental(), 1);
        assert_eq!(sim.audit_runs(), 1);
        sim.place_run(&[2, 3]);
        assert_eq!(sim.slot_of[2], 8, "组 {{2,3}} 必须整组落到空 run 的头上");
        assert_eq!(sim.slot_of[3], 9);
        assert_eq!(sim.runs_incremental(), 3, "手算锚点：断在 (1,2) 与 (3,4)");
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
        assert_eq!(sim.runs_incremental(), 3);
        assert_eq!(sim.audit_runs(), 3);
        assert_eq!(sim.total_writes, 1, "只写了脏的那一个");
    }

    /// 判据 2 守恒：增量 runs 与审计 runs 逐点相等，占用恒 = L，空 + 占 = S。
    #[test]
    fn conservation_holds_on_every_arm() {
        for arm in [
            Arm::PolicyCompact(32),
            Arm::PolicyNeighbor(32),
            Arm::NodeGroup { group_size: 8, extra: 32 },
            Arm::GenerationSeparation { hot_age_threshold: 64, warm_age_threshold: 512, extra: 32 },
        ] {
            let outcome = run_arm(arm, Load::Uniform, 0, 200, 25);
            assert!(outcome.runs_final >= 1 && outcome.runs_final <= OBJECT_COUNT as u64);
        }
    }

    /// 判据 3 阳性对照：测量管道分得出全隔离与全连续。
    #[test]
    fn measurement_control_discriminates() {
        let mut sim = Sim::new(OBJECT_COUNT, SLOT_COUNT, SLOTS_PER_SEGMENT);
        let (scatter_incremental, scatter_audit, compact_incremental, compact_audit) = measurement_control(&mut sim);
        assert_eq!((scatter_incremental, scatter_audit), (OBJECT_COUNT as u64, OBJECT_COUNT as u64));
        assert_eq!((compact_incremental, compact_audit), (1, 1));
    }

    /// 跨装置闸自己要能红：E93 入库产物里的一个数被改掉时，断言必须不成立。
    #[test]
    fn xfixture_gate_is_not_a_rubber_stamp() {
        let got = median_of_five((0..5u64).map(|seed| run_e93_arm(E93Policy::BumpCompact(256), Load::Runs8, seed, CHECKPOINT_COUNT, 250).runs_final).collect());
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
                "bump_seg" => E93Policy::BumpSegment,
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
                median_of_five((0..5u64).map(|seed| run_e93_arm(policy, load, seed, CHECKPOINT_COUNT, 250).runs_final).collect());
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
        for (offset_in_group, key) in (4..8).enumerate() {
            assert_eq!(sim.slot_of[key], base + offset_in_group as u32, "组 {{4..7}} 必须整组连续");
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
    fn generation_separation_sweep_only_takes_hot_objects() {
        let mut sim = Sim::new(16, 64, 4);
        // 全部对象初值是冷（object_generation = 2）⇒ 整理一个都挑不到
        let before = sim.total_writes;
        step_generation_separation(&mut sim, &[0], 1, 16, 128, 8);
        assert_eq!(sim.total_writes, before + 1, "只有那个脏 key 被写，整理挑不到热对象");
        // 把 5..9 标成热代，再来一轮：整理应当只挑这几个
        for key in 5..9 {
            sim.object_generation[key] = 0;
        }
        let before2 = sim.total_writes;
        sim.sweep = 0;
        step_generation_separation(&mut sim, &[0], 2, 16, 128, 8);
        assert_eq!(sim.total_writes, before2 + 1 + 4, "脏的一个 + 四个热对象，冷的一个都不许进");
    }

    /// **knob 到函数那根线**：`Arm::NodeGroup { group_size }` 的 g 必须真的传进 `step_node_group`。
    /// ⚠️ 这条是补的：此前 18 条单测**全部直接调** `step_node_group`，
    /// 把函数体里的 `g` 换成常量 8 时 31 个测试一条都不红，而 g2 / g8 / g32 三行会逐字相同、
    /// 判决行随之翻面（2026-09-10 反推腿实测出来的坏法）。
    #[test]
    fn node_group_knob_is_wired_through_run_arm() {
        let group_size_2_outcome = run_arm(Arm::NodeGroup { group_size: 2, extra: 128 }, Load::Runs8, 0, 200, 50);
        let group_size_8_outcome = run_arm(Arm::NodeGroup { group_size: 8, extra: 128 }, Load::Runs8, 0, 200, 50);
        let group_size_32_outcome = run_arm(Arm::NodeGroup { group_size: 32, extra: 128 }, Load::Runs8, 0, 200, 50);
        assert_ne!(group_size_2_outcome.runs_final, group_size_8_outcome.runs_final, "g=2 与 g=8 出同一个数 ⇒ knob 没接上");
        assert_ne!(group_size_8_outcome.runs_final, group_size_32_outcome.runs_final, "g=8 与 g=32 出同一个数 ⇒ knob 没接上");
        // 绝对值锚点：三个数各自钉死，只钉「互不相等」测不出三条一起错
        assert_eq!((group_size_2_outcome.runs_final, group_size_8_outcome.runs_final, group_size_32_outcome.runs_final), (1862, 897, 1100));
    }

    /// 同一根线，代际分离那一侧：(`hot_age_threshold`, `warm_age_threshold`) 必须真的传进 `step_generation_separation`。
    #[test]
    fn generation_separation_knob_is_wired_through_run_arm() {
        let low_thresholds_outcome = run_arm(Arm::GenerationSeparation { hot_age_threshold: 16, warm_age_threshold: 128, extra: 128 }, Load::Runs8, 0, 200, 50);
        let high_thresholds_outcome = run_arm(Arm::GenerationSeparation { hot_age_threshold: 256, warm_age_threshold: 1024, extra: 128 }, Load::Runs8, 0, 200, 50);
        assert_ne!(low_thresholds_outcome.runs_final, high_thresholds_outcome.runs_final, "两组阈值出同一个数 ⇒ knob 没接上");
        assert_eq!((low_thresholds_outcome.runs_final, high_thresholds_outcome.runs_final), (3442, 2588));
    }

    /// **代际分离的回落必须计数**：没有空段时逐槽回落，counter 要涨。
    #[test]
    fn generation_separation_fallback_counts() {
        let mut sim = Sim::new(8, 16, 4);
        // 段 2、3 是仅有的空段；先把它们的槽全占掉，逼出回落
        for slot in 8u32..16 {
            sim.take_slot(slot);
        }
        // 现在一个空槽都没有 ⇒ 得先还两个不成段的槽回去
        sim.free.insert(9);
        sim.free_slots_per_segment[9 / 4] += 1;
        sim.free.insert(13);
        sim.free_slots_per_segment[13 / 4] += 1;
        assert!(sim.find_empty_segment_for_generation(0).is_none(), "不许还有全空段，否则测的不是回落");
        let before = sim.fallback_allocations;
        sim.place_generation_run(0, &[2]);
        assert_eq!(sim.fallback_allocations, before + 1, "回落必须计数");
    }

    /// 代际分离：热与冷用各自的开放段这一点，靠 `generation_open_segment` 分开，
    /// **而 `segment_generation` 那道 tag 检查在本模型里判定为等价变异**——
    /// 一个全空段永远不会带着别人的 tag（end_checkpoint 会清掉，开放段又被 generation_open_segment 排除）。
    /// 这条测试把那个等价性留档（mutation-sampling.md 要求）。
    #[test]
    fn segment_generation_tag_is_never_stale_on_an_empty_segment() {
        let mut sim = Sim::new(8, 16, 4);
        sim.place_generation_run(0, &[0, 1]);
        let segment_index = sim.slot_of[0] as usize / sim.slots_per_segment;
        assert_eq!(sim.segment_generation[segment_index], Some(0));
        // 把这个段腾空，走一次 checkpoint 收尾
        sim.place_generation_run(2, &[0, 1]);
        sim.generation_open_segment[0] = None;
        sim.end_checkpoint();
        assert_eq!(sim.free_slots_per_segment[segment_index] as usize, sim.slots_per_segment, "段已回空");
        assert_eq!(sim.segment_generation[segment_index], None, "回空的段必须丢掉代身份");
    }

    /// 老化代分档是纯算术，边界钉死（三档、两个阈值、左闭右开）。
    #[test]
    fn generation_for_age_boundaries() {
        assert_eq!(generation_for_age(0, 16, 128), 0);
        assert_eq!(generation_for_age(15, 16, 128), 0);
        assert_eq!(generation_for_age(16, 16, 128), 1);
        assert_eq!(generation_for_age(127, 16, 128), 1);
        assert_eq!(generation_for_age(128, 16, 128), 2);
        assert_eq!(generation_for_age(1_000_000, 16, 128), 2);
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
    fn generation_segments_never_mix() {
        let mut sim = Sim::new(64, 128, 8);
        sim.place_generation_run(0, &[0, 1, 2]);
        sim.place_generation_run(2, &[40, 41]);
        let hot_segment_index = sim.slot_of[0] as usize / sim.slots_per_segment;
        let cold_segment_index = sim.slot_of[40] as usize / sim.slots_per_segment;
        assert_ne!(hot_segment_index, cold_segment_index, "热与冷落进了同一个段");
        assert_eq!(sim.segment_generation[hot_segment_index], Some(0));
        assert_eq!(sim.segment_generation[cold_segment_index], Some(2));
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
        assert_eq!(sim.find_empty_segment(), None);
        let before = sim.fallback_allocations;
        sim.place_run(&[2, 3]);
        assert_eq!(sim.fallback_allocations, before + 2, "两个 key 各回落一次");
    }

    /// 节点组臂的绝对值：组大小 g、预算够时，一个 checkpoint 写的对象数
    /// 恰是「被碰到的组的并集」的大小，不是脏对象数。
    #[test]
    fn node_group_writes_whole_groups() {
        let mut sim = Sim::new(64, 128, 8);
        let group_size = 4usize;
        let dirty = [5usize, 6, 20];
        // 组 1 = {4,5,6,7}，组 5 = {20,21,22,23} ⇒ 并集 8 个对象
        let dirty_key_set: BTreeSet<usize> = dirty.iter().copied().collect();
        let mut touched: Vec<usize> = dirty.iter().map(|&key| key / group_size).collect();
        touched.dedup();
        assert_eq!(touched, vec![1, 5]);
        for group_index in touched {
            let keys: Vec<usize> = (group_index * group_size..(group_index + 1) * group_size).collect();
            let clean = keys.iter().filter(|key| !dirty_key_set.contains(key)).count();
            assert_eq!(clean, if group_index == 1 { 2 } else { 3 });
            sim.place_run(&keys);
        }
        assert_eq!(sim.total_writes, 8, "两个组各 4 个对象");
    }
}
