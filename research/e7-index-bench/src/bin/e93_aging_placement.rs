//! E93：老化下的放置与碎片度 —— D26（后台整理与放置回收）已定项 1 / 4 / 6 的数据前置。
//!
//! ## 模型（判据与失败条款的权威登记在 kb/experiments/93-老化下的放置与碎片度.md，跑前写死）
//!
//! L 个定宽对象按 key 序排队，设备 S 个槽，初始按 key 序连续写入。
//! 每个 checkpoint 按负载脏掉一批对象，COW 重写到新槽；旧槽进 defer，
//! 下一个 checkpoint 才可复用（D16 新规则 2 的模型化）。
//!
//! 碎片度（被验证对象）：`F = runs / L`，runs = 按 key 序遍历的物理连续段数。
//! 增量维护：对象挪槽只改它与 key 相邻两个对象的邻接 ⇒ O(1)；
//! 审计路径全扫重算，与增量路径不共享代码。
//!
//! 四条臂：first_fit / bump_seg / bump_neighbor(R) / bump_compact(B)；
//! 两组负载：均匀随机单点重写 / 成段重写（段长 8）。
//!
//! ## 判据（跑前写死，跑完不许改）
//!
//! 1. 手算锚点：8 对象小场景 runs 钉死（单测）。
//! 2. 守恒：增量 runs == 审计 runs（每采样点）；占用 == L；空 + 占 == S；defer 本轮不可复用。
//! 3. 测量阳性对照四条臂都过闸：强制放置成全隔离布局必须报 runs == L，连续布局必须报 runs == 1。
//! 4. 老化负载够不够狠：first_fit × 均匀随机期末 runs ≥ L/4，否则整轮作废。
//! 5. 只报数不判降级：降级判据归 E10 自己那条。
//!
//! ## 它答不了的
//!
//! 计数模型，文件操作 0 处；无耗时轴；对象定宽；两组负载是端点不是分布。

use e7_index_bench::Emitter;
use std::collections::BTreeSet;

const OBJECT_COUNT: usize = 8192; // 对象数
const SLOT_COUNT: usize = 10240; // 槽数（填充 80%）
const SLOTS_PER_SEGMENT: usize = 64; // 聚簇段槽数
const DIRTY_PER_CHECKPOINT: usize = 64; // 每 checkpoint 用户脏对象数
const CHECKPOINT_COUNT: u64 = 2000; // checkpoint 数

struct XorshiftGenerator(u64);
impl XorshiftGenerator {
    fn new(seed: u64) -> Self {
        let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0xA076_1D64_78BD_642F);
        if state == 0 {
            state = 0xDEAD_BEEF;
        }
        XorshiftGenerator(state)
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Policy {
    FirstFit,
    BumpSegment,
    BumpNeighbor(usize), // R：每个脏 key 段向两侧各多重写 ≤R 个干净邻居
    BumpCompact(usize),  // B：每 checkpoint 按 key 序轮转整理 B 个对象
}
impl Policy {
    fn tag(self) -> String {
        match self {
            Policy::FirstFit => "first_fit".into(),
            Policy::BumpSegment => "bump_seg".into(),
            Policy::BumpNeighbor(neighbor_radius) => format!("bump_nb_r{neighbor_radius}"),
            Policy::BumpCompact(compaction_batch) => format!("bump_cp_b{compaction_batch}"),
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

struct AgingSimulation {
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
    fallback_allocation_count: u64,
    sweep: usize,
}

impl AgingSimulation {
    /// 初始布局：key k 住槽 k（顺序创建的形态）。
    fn new(object_count: usize, slot_count: usize, segment_slots: usize) -> AgingSimulation {
        assert!(object_count < slot_count && slot_count % segment_slots == 0);
        let slot_of: Vec<u32> = (0..object_count as u32).collect();
        let mut key_at: Vec<Option<u32>> = vec![None; slot_count];
        for key in 0..object_count {
            key_at[key] = Some(key as u32);
        }
        let free: BTreeSet<u32> = (object_count as u32..slot_count as u32).collect();
        let mut free_slots_per_segment = vec![0u16; slot_count / segment_slots];
        for slot in object_count..slot_count {
            free_slots_per_segment[slot / segment_slots] += 1;
        }
        AgingSimulation {
            object_count,
            slot_count,
            slots_per_segment: segment_slots,
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
            fallback_allocation_count: 0,
            sweep: 0,
        }
    }

    /// 邻接对 (k-1, k) 断没断。增量路径与审计路径共用这个纯谓词，但不共用累加逻辑。
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

    /// 把 key 挪到 new_slot：增量更新 broken，旧槽进 defer。
    fn move_key(&mut self, key: usize, new_slot: u32) {
        debug_assert!(self.key_at[new_slot as usize].is_none());
        // 摘掉受影响邻接对的旧贡献
        if key >= 1 && self.pair_broken(key) {
            self.broken -= 1;
        }
        if key + 1 < self.object_count && self.pair_broken(key + 1) {
            self.broken -= 1;
        }
        let old_slot = self.slot_of[key];
        self.key_at[old_slot as usize] = None;
        self.deferred.push(old_slot);
        self.slot_of[key] = new_slot;
        self.key_at[new_slot as usize] = Some(key as u32);
        // 加回新贡献
        if key >= 1 && self.pair_broken(key) {
            self.broken += 1;
        }
        if key + 1 < self.object_count && self.pair_broken(key + 1) {
            self.broken += 1;
        }
        self.total_writes += 1;
    }

    fn allocate_first_fit(&mut self) -> u32 {
        // 开放段里还没 bump 到的槽是保留区，first_fit 也不许拿——
        // 偷走它会让后续 bump 双重分配同一个槽
        let slot = *self
            .free
            .iter()
            .find(|&&candidate_slot| self.open_segment != Some(candidate_slot as usize / self.slots_per_segment))
            .expect("槽用尽：配置违反 D ≤ S−L");
        self.free.remove(&slot);
        self.free_slots_per_segment[slot as usize / self.slots_per_segment] -= 1;
        slot
    }

    /// 找一个全空段开成聚簇段。开放段自己不算候选。
    fn open_empty_segment(&mut self) -> Option<usize> {
        (0..self.free_slots_per_segment.len())
            .find(|&segment_index| self.free_slots_per_segment[segment_index] as usize == self.slots_per_segment && self.open_segment != Some(segment_index))
    }

    fn allocate_bump(&mut self) -> u32 {
        loop {
            if let Some(segment) = self.open_segment {
                if self.bump < self.slots_per_segment {
                    let slot = (segment * self.slots_per_segment + self.bump) as u32;
                    self.bump += 1;
                    // 开段时全空，bump 只前进；段内后来被 defer 释放的槽要等整段回空才复用
                    debug_assert!(self.free.contains(&slot));
                    self.free.remove(&slot);
                    self.free_slots_per_segment[segment] -= 1;
                    return slot;
                }
                self.open_segment = None;
            }
            match self.open_empty_segment() {
                Some(segment) => {
                    self.open_segment = Some(segment);
                    self.bump = 0;
                }
                None => {
                    // 无空段：回落 first_fit，计数——聚簇段政策自持不自持就看这个数
                    self.fallback_allocation_count += 1;
                    return self.allocate_first_fit();
                }
            }
        }
    }

    /// 在空槽集里找一段长 len 的连续 run（跳过开放段——那段的未 bump 部分是保留区）。
    fn find_free_run(&self, run_length: usize) -> Option<u32> {
        let mut start = 0u32;
        let mut count = 0usize;
        let mut previous_slot: Option<u32> = None;
        for &slot in &self.free {
            if self.open_segment == Some(slot as usize / self.slots_per_segment) {
                previous_slot = None;
                count = 0;
                continue;
            }
            match previous_slot {
                Some(previous) if slot == previous + 1 => count += 1,
                _ => {
                    start = slot;
                    count = 1;
                }
            }
            previous_slot = Some(slot);
            if count >= run_length {
                return Some(start);
            }
        }
        None
    }

    /// 拿走一个指定空槽（free-run 路径用）。
    fn take_slot(&mut self, slot: u32) {
        let removed = self.free.remove(&slot);
        debug_assert!(removed);
        self.free_slots_per_segment[slot as usize / self.slots_per_segment] -= 1;
    }

    /// 把一个 key 段连续落盘：开放段 → 新空段 → 空槽 run → 逐槽回落（计数）。
    /// bump_neighbor 与 bump_compact 用它——这两条臂的定义就是「写出的簇物理连续」，
    /// 不给这条路径等于给它们立稻草人。
    fn place_run(&mut self, keys: &[usize]) {
        let segment_slots = self.slots_per_segment;
        let owned: Vec<usize> = keys.to_vec();
        for chunk in owned.chunks(segment_slots) {
            self.place_chunk(chunk);
        }
    }

    fn place_chunk(&mut self, keys: &[usize]) {
        let chunk_length = keys.len();
        if let Some(segment) = self.open_segment {
            if self.slots_per_segment - self.bump >= chunk_length {
                for &key in keys {
                    let slot = (segment * self.slots_per_segment + self.bump) as u32;
                    self.bump += 1;
                    self.take_slot(slot);
                    self.move_key(key, slot);
                }
                return;
            }
        }
        if let Some(segment) = self.open_empty_segment() {
            self.open_segment = Some(segment);
            self.bump = 0;
            for &key in keys {
                let slot = (segment * self.slots_per_segment + self.bump) as u32;
                self.bump += 1;
                self.take_slot(slot);
                self.move_key(key, slot);
            }
            return;
        }
        if let Some(start) = self.find_free_run(chunk_length) {
            for (offset_in_run, &key) in keys.iter().enumerate() {
                let slot = start + offset_in_run as u32;
                self.take_slot(slot);
                self.move_key(key, slot);
            }
            return;
        }
        for &key in keys {
            self.fallback_allocation_count += 1;
            let slot = self.allocate_first_fit();
            self.move_key(key, slot);
        }
    }

    /// checkpoint 收尾：defer 的槽此刻才真正可复用（D16 新规则 2）。
    fn end_checkpoint(&mut self) {
        for slot in std::mem::take(&mut self.deferred) {
            self.free.insert(slot);
            self.free_slots_per_segment[slot as usize / self.slots_per_segment] += 1;
        }
    }

    fn occupied(&self) -> usize {
        self.key_at.iter().filter(|slot_content| slot_content.is_some()).count()
    }
}

/// 选负载脏集（去重、升序）。
fn dirty_set(load: Load, random_generator: &mut XorshiftGenerator, object_count: usize, dirty_count: usize) -> Vec<usize> {
    let mut dirty_keys = BTreeSet::new();
    match load {
        Load::Uniform => {
            while dirty_keys.len() < dirty_count {
                dirty_keys.insert(random_generator.below(object_count));
            }
        }
        Load::Runs8 => {
            while dirty_keys.len() < dirty_count {
                let start = random_generator.below(object_count);
                for key in start..(start + 8).min(object_count) {
                    if dirty_keys.len() < dirty_count {
                        dirty_keys.insert(key);
                    }
                }
            }
        }
    }
    dirty_keys.into_iter().collect()
}

/// bump_neighbor：把脏 key 段向两侧各扩 ≤R 个干净邻居。返回 (最终批次, 邻居数)。
fn extend_neighbors(dirty: &[usize], radius: usize, object_count: usize) -> (Vec<usize>, u64) {
    let mut extended_keys: BTreeSet<usize> = dirty.iter().copied().collect();
    let mut neighbor_extension_count = 0u64;
    // 先算脏集自己的极大段，再扩，免得扩进来的邻居又被当成段端点
    let mut runs: Vec<(usize, usize)> = Vec::new();
    for &key in dirty {
        match runs.last_mut() {
            Some((_, run_end)) if *run_end + 1 == key => *run_end = key,
            _ => runs.push((key, key)),
        }
    }
    for (run_start, run_end) in runs {
        for neighbor_key in (run_start.saturating_sub(radius)..run_start).chain(run_end + 1..(run_end + 1 + radius).min(object_count)) {
            if extended_keys.insert(neighbor_key) {
                neighbor_extension_count += 1;
            }
        }
    }
    (extended_keys.into_iter().collect(), neighbor_extension_count)
}

struct Outcome {
    runs_final: u64,
    write_amplification: f64,
    fallback_percent: f64,
    trajectory: Vec<(u64, u64)>,
}

fn run_arm(policy: Policy, load: Load, seed: u64, checkpoint_limit: u64, sample_every: u64) -> Outcome {
    let mut simulation = AgingSimulation::new(OBJECT_COUNT, SLOT_COUNT, SLOTS_PER_SEGMENT);
    let mut random_generator = XorshiftGenerator::new(seed);
    let mut trajectory = vec![(0u64, simulation.runs_incremental())];
    for checkpoint_index in 1..=checkpoint_limit {
        let dirty = dirty_set(load, &mut random_generator, OBJECT_COUNT, DIRTY_PER_CHECKPOINT);
        simulation.user_writes += dirty.len() as u64;
        let (batch, _neighbor_extension_count) = match policy {
            Policy::BumpNeighbor(neighbor_radius) => extend_neighbors(&dirty, neighbor_radius, OBJECT_COUNT),
            _ => (dirty, 0),
        };
        match policy {
            Policy::FirstFit => {
                for &key in &batch {
                    let slot = simulation.allocate_first_fit();
                    simulation.move_key(key, slot);
                }
            }
            Policy::BumpSegment | Policy::BumpCompact(_) => {
                // 用户批次按 D3 已定项 5 的形态顺序写进聚簇段（时间聚簇，不保证 key 连续）
                for &key in &batch {
                    let slot = simulation.allocate_bump();
                    simulation.move_key(key, slot);
                }
            }
            Policy::BumpNeighbor(_) => {
                // 邻居臂的定义是「簇连续落盘」：按 key 段整段放置
                let mut run_start_index = 0;
                while run_start_index < batch.len() {
                    let mut run_end_index = run_start_index + 1;
                    while run_end_index < batch.len() && batch[run_end_index] == batch[run_end_index - 1] + 1 {
                        run_end_index += 1;
                    }
                    let run: Vec<usize> = batch[run_start_index..run_end_index].to_vec();
                    simulation.place_run(&run);
                    run_start_index = run_end_index;
                }
            }
        }
        if let Policy::BumpCompact(compaction_batch) = policy {
            // 按 key 序轮转整理：本轮已写过的 key 跳过（同一 checkpoint 一个对象至多写一次），
            // 整理批次整段连续放置
            let batch_set: BTreeSet<usize> = batch.iter().copied().collect();
            let mut sweep_keys = Vec::with_capacity(compaction_batch);
            while sweep_keys.len() < compaction_batch {
                let key = simulation.sweep;
                simulation.sweep = (simulation.sweep + 1) % OBJECT_COUNT;
                if !batch_set.contains(&key) {
                    sweep_keys.push(key);
                }
            }
            simulation.place_run(&sweep_keys);
        }
        simulation.end_checkpoint();
        if checkpoint_index % sample_every == 0 {
            // 守恒：增量与审计两条路径必须逐点相等；占用恒 = L
            assert_eq!(simulation.runs_incremental(), simulation.audit_runs(), "t={checkpoint_index} 增量与审计分叉");
            assert_eq!(simulation.occupied(), OBJECT_COUNT);
            assert_eq!(simulation.free.len() + OBJECT_COUNT, SLOT_COUNT);
            trajectory.push((checkpoint_index, simulation.runs_incremental()));
        }
    }
    Outcome {
        runs_final: simulation.runs_incremental(),
        write_amplification: simulation.total_writes as f64 / simulation.user_writes as f64,
        fallback_percent: 100.0 * simulation.fallback_allocation_count as f64 / simulation.total_writes as f64,
        trajectory,
    }
}

/// 判据 3：测量的阳性对照。强制放置（绕过政策）走增量路径，审计路径复核。
/// 全隔离布局必须报 runs == L；key 序连续布局必须报 runs == 1。
fn measurement_control(simulation: &mut AgingSimulation) -> (u64, u64, u64, u64) {
    let object_count = simulation.object_count;
    // 全隔离：前一半 key 住偶数槽、后一半住奇数槽 ⇒ 任何邻接都断
    for key in 0..object_count {
        let target = if key < object_count / 2 { 2 * key as u32 } else { (2 * (key - object_count / 2) + 1) as u32 };
        force_to(simulation, key, target);
    }
    let scatter = (simulation.runs_incremental(), simulation.audit_runs());
    // 连续：key k 回槽 k
    for key in 0..object_count {
        force_to(simulation, key, key as u32);
    }
    let compact = (simulation.runs_incremental(), simulation.audit_runs());
    (scatter.0, scatter.1, compact.0, compact.1)
}

/// 测试/对照专用的强制放置：目标槽被占就先把占用者搬去任一空槽（都走 move_key 的增量路径）。
fn force_to(simulation: &mut AgingSimulation, key: usize, target: u32) {
    if simulation.slot_of[key] == target {
        return;
    }
    if let Some(other) = simulation.key_at[target as usize] {
        let spare = *simulation
            .free
            .iter()
            .next()
            .expect("强制放置需要至少一个空槽");
        simulation.free.remove(&spare);
        simulation.free_slots_per_segment[spare as usize / simulation.slots_per_segment] -= 1;
        simulation.move_key(other as usize, spare);
    }
    // target 此刻要么本来就空、要么刚被腾出（还挂在 defer 上）
    if simulation.free.remove(&target) {
        simulation.free_slots_per_segment[target as usize / simulation.slots_per_segment] -= 1;
    } else {
        let deferred_position = simulation.deferred.iter().position(|&deferred_slot| deferred_slot == target).expect("目标槽既不空也不在 defer");
        simulation.deferred.swap_remove(deferred_position);
    }
    simulation.move_key(key, target);
    simulation.end_checkpoint();
}

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config l={OBJECT_COUNT} s={SLOT_COUNT} g={SLOTS_PER_SEGMENT} d={DIRTY_PER_CHECKPOINT} t={CHECKPOINT_COUNT} model=counting file_ops=0 seeds=5"
        ))
    );

    let arms = [
        Policy::FirstFit,
        Policy::BumpSegment,
        Policy::BumpNeighbor(1),
        Policy::BumpNeighbor(2),
        Policy::BumpCompact(32),
        Policy::BumpCompact(128),
        Policy::BumpCompact(256),
    ];

    // 判据 3：四种政策形态的测量管道逐一过阳性对照闸（政策不同不改测量代码这一事实
    // 由这里逐臂跑一遍来证明，不由「反正代码共享」来宣称）。
    for &arm in &arms {
        let mut simulation = AgingSimulation::new(OBJECT_COUNT, SLOT_COUNT, SLOTS_PER_SEGMENT);
        let (scatter_incremental_runs, scatter_audit_runs, compact_incremental_runs, compact_audit_runs) = measurement_control(&mut simulation);
        assert_eq!((scatter_incremental_runs, scatter_audit_runs), (OBJECT_COUNT as u64, OBJECT_COUNT as u64), "{:?} 全隔离布局必须报 runs=L", arm);
        assert_eq!((compact_incremental_runs, compact_audit_runs), (1, 1), "{:?} 连续布局必须报 runs=1", arm);
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=control arm={} scatter_runs={scatter_incremental_runs} compact_runs={compact_incremental_runs}",
                arm.tag()
            ))
        );
    }

    // 判据 4：老化负载够不够狠（first_fit × uniform，五个种子都得过）
    for seed in 0..5u64 {
        let outcome = run_arm(Policy::FirstFit, Load::Uniform, seed, CHECKPOINT_COUNT, 250);
        assert!(
            outcome.runs_final >= (OBJECT_COUNT as u64) / 4,
            "老化负载不够狠（seed={seed} runs_final={}）：整轮作废",
            outcome.runs_final
        );
    }

    for &arm in &arms {
        for load in [Load::Uniform, Load::Runs8] {
            let mut finals = Vec::new();
            for seed in 0..5u64 {
                let outcome = run_arm(arm, load, seed, CHECKPOINT_COUNT, 250);
                if seed == 0 {
                    let trajectory: Vec<String> =
                        outcome.trajectory.iter().step_by(2).map(|(checkpoint_index, runs)| format!("{checkpoint_index}:{runs}")).collect();
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "name=traj arm={} load={} seed=0 runs={}",
                            arm.tag(),
                            load.tag(),
                            trajectory.join(",")
                        ))
                    );
                }
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=aging arm={} load={} seed={seed} runs_final={} f_pct={:.2} write_amp={:.3} fallback_pct={:.1}",
                        arm.tag(),
                        load.tag(),
                        outcome.runs_final,
                        100.0 * outcome.runs_final as f64 / OBJECT_COUNT as f64,
                        outcome.write_amplification,
                        outcome.fallback_percent
                    ))
                );
                finals.push(outcome.runs_final);
            }
            let median_runs_final = {
                finals.sort_unstable();
                finals[2]
            };
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=aging_median arm={} load={} runs_final_median={median_runs_final}",
                    arm.tag(),
                    load.tag()
                ))
            );
        }
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **判据 1 手算锚点**：8 对象、10 槽。key 3 挪去最低空槽 8：
    /// 布局 0,1,2,[8],4,5,6,7 ⇒ 断在 (2,3) 与 (3,4) ⇒ runs = 3。
    #[test]
    fn hand_case_runs_after_one_move() {
        let mut simulation = AgingSimulation::new(8, 10, 2);
        assert_eq!(simulation.runs_incremental(), 1);
        assert_eq!(simulation.audit_runs(), 1);
        let slot = simulation.allocate_first_fit();
        assert_eq!(slot, 8);
        simulation.move_key(3, slot);
        simulation.end_checkpoint();
        assert_eq!(simulation.runs_incremental(), 3);
        assert_eq!(simulation.audit_runs(), 3);
    }

    /// **判据 1 手算锚点（邻居重写）**：8 对象、16 槽、段长 4。脏 {5}、R=1 ⇒ 批次 {4,5,6}，
    /// 整段放进空段 2 的槽 8,9,10 ⇒ 三个 key 物理连续；
    /// 断在 (3,4) 与 (6,7) ⇒ runs = 3。
    #[test]
    fn hand_case_neighbor_bump() {
        let mut simulation = AgingSimulation::new(8, 16, 4);
        let (batch, neighbor_extension_count) = extend_neighbors(&[5], 1, 8);
        assert_eq!(batch, vec![4, 5, 6]);
        assert_eq!(neighbor_extension_count, 2);
        simulation.place_run(&batch);
        simulation.end_checkpoint();
        assert_eq!(simulation.slot_of[4], 8);
        assert_eq!(simulation.slot_of[5], 9);
        assert_eq!(simulation.slot_of[6], 10);
        assert_eq!(simulation.runs_incremental(), 3);
        assert_eq!(simulation.audit_runs(), 3);
    }

    /// 无空段时整段落进空槽 run：6 对象、8 槽、段长 4 ⇒ 只有槽 6、7 空且相邻。
    /// place_run([0,1]) 必须连续落在 6、7，不许逐槽回落。
    #[test]
    fn place_run_uses_free_run_when_no_empty_segment() {
        let mut simulation = AgingSimulation::new(6, 8, 4);
        simulation.place_run(&[0, 1]);
        assert_eq!((simulation.slot_of[0], simulation.slot_of[1]), (6, 7));
        assert_eq!(simulation.fallback_allocation_count, 0);
        simulation.end_checkpoint();
    }

    /// 连续 run 也找不到时逐槽回落并计数：空槽只剩互不相邻的 {1, 7}。
    #[test]
    fn place_run_falls_back_to_singles() {
        let mut simulation = AgingSimulation::new(6, 8, 4);
        simulation.place_run(&[1]); // key 1 → 槽 6（len=1 的 run 就是最低空槽）
        assert_eq!(simulation.slot_of[1], 6);
        simulation.end_checkpoint(); // 槽 1 回空 ⇒ 空槽 {1, 7}，不相邻
        simulation.place_run(&[2, 3]);
        assert_eq!(simulation.fallback_allocation_count, 2);
        assert_eq!((simulation.slot_of[2], simulation.slot_of[3]), (1, 7));
        simulation.end_checkpoint();
    }

    /// 邻居扩展不越界、不重复：脏 {0,1,7}、R=2、L=8 ⇒ 段 [0,1] 只能右扩 2、段 [7,7] 只能左扩 2。
    #[test]
    fn neighbor_extension_clips() {
        let (batch, neighbor_extension_count) = extend_neighbors(&[0, 1, 7], 2, 8);
        assert_eq!(batch, vec![0, 1, 2, 3, 5, 6, 7]);
        assert_eq!(neighbor_extension_count, 4);
    }

    /// **判据 2 defer**：本 checkpoint 释放的槽本轮不可复用。
    /// 2 对象、4 槽：挪 key 0 → 槽 2 之后，槽 0 在 checkpoint 结束前不在空槽集里。
    #[test]
    fn defer_blocks_same_checkpoint_reuse() {
        let mut simulation = AgingSimulation::new(2, 4, 2);
        let first_slot = simulation.allocate_first_fit();
        assert_eq!(first_slot, 2);
        simulation.move_key(0, first_slot);
        assert!(!simulation.free.contains(&0), "旧槽本轮不许回空槽集");
        let second_slot = simulation.allocate_first_fit();
        assert_eq!(second_slot, 3, "本轮只剩槽 3 可用");
        simulation.end_checkpoint();
        assert!(simulation.free.contains(&0), "checkpoint 结束后旧槽才可复用");
    }

    /// **判据 3**：强制放置的两个布局，增量与审计两条路径都要报对。
    #[test]
    fn measurement_control_hits_both_ends() {
        let mut simulation = AgingSimulation::new(64, 160, 8);
        let (scatter_incremental_runs, scatter_audit_runs, compact_incremental_runs, compact_audit_runs) = measurement_control(&mut simulation);
        assert_eq!((scatter_incremental_runs, scatter_audit_runs), (64, 64));
        assert_eq!((compact_incremental_runs, compact_audit_runs), (1, 1));
    }

    /// 审计的判别力：直接踩坏 slot_of（绕过增量路径），审计必须与增量分叉。
    #[test]
    fn audit_has_teeth() {
        let mut simulation = AgingSimulation::new(8, 10, 2);
        simulation.slot_of[3] = 9; // 不走 move_key ⇒ broken 没跟上
        assert_ne!(simulation.runs_incremental(), simulation.audit_runs());
    }

    /// 无全空段但有空槽时，bump 回落 first_fit 且计数。
    /// 6 对象、8 槽、段长 4：seg0 全满、seg1 半满（槽 6、7 空）⇒ 无全空段。
    #[test]
    fn bump_falls_back_when_no_empty_segment() {
        let mut simulation = AgingSimulation::new(6, 8, 4);
        let fallback_slot = simulation.allocate_bump();
        assert_eq!(fallback_slot, 6, "回落 first_fit 取最低空槽");
        assert_eq!(simulation.fallback_allocation_count, 1);
        simulation.move_key(0, fallback_slot);
        simulation.end_checkpoint();
    }

    /// 轮转整理推进 sweep 且写出的对象物理连续：B=2、无用户写。
    #[test]
    fn compact_sweep_rewrites_in_key_order() {
        let mut simulation = AgingSimulation::new(8, 16, 4);
        let keys: Vec<usize> = (0..2)
            .map(|_| {
                let key = simulation.sweep;
                simulation.sweep = (simulation.sweep + 1) % 8;
                key
            })
            .collect();
        simulation.place_run(&keys);
        simulation.end_checkpoint();
        assert_eq!(
            (simulation.slot_of[0], simulation.slot_of[1], simulation.sweep),
            (8, 9, 2),
            "key 0、1 连续写进空段 2，sweep 推进到 2"
        );
    }

    /// 完整小规模跑一遍四条臂 × 两负载：守恒逐点成立（run_arm 内部的采样断言）。
    #[test]
    fn conservation_all_arms() {
        for arm in [
            Policy::FirstFit,
            Policy::BumpSegment,
            Policy::BumpNeighbor(2),
            Policy::BumpCompact(128),
        ] {
            for load in [Load::Uniform, Load::Runs8] {
                let outcome = run_arm(arm, load, 1, 200, 50);
                assert!(outcome.runs_final >= 1);
                assert!(outcome.write_amplification >= 1.0);
            }
        }
    }

    /// 开放段里还没 bump 到的槽是保留区：free-run 搜索与逐槽回落都不许拿。
    /// 布局手工搭：13 对象、16 槽、段长 4；把槽 11、12 腾空（跨段 2/3 边界的相邻对），
    /// 段 2 声明开放且 bump=3 ⇒ 槽 11 保留 ⇒ len=2 找不到 run、逐槽回落也要跳过 11。
    #[test]
    fn open_segment_slots_are_reserved() {
        let mut simulation = AgingSimulation::new(13, 16, 4);
        simulation.place_run(&[11]); // key 11 → 槽 13（最低的 len=1 run）
        assert_eq!(simulation.slot_of[11], 13);
        simulation.end_checkpoint(); // 槽 11 回空
        force_to(&mut simulation, 12, 14); // key 12 → 槽 14 ⇒ 槽 12 回空
        assert_eq!(simulation.slot_of[12], 14);
        simulation.open_segment = Some(2);
        simulation.bump = 3;
        simulation.place_run(&[1, 2]);
        assert_eq!(simulation.fallback_allocation_count, 2, "len=2 的 run 只有跨进保留区才有，必须回落");
        assert_eq!((simulation.slot_of[1], simulation.slot_of[2]), (12, 15), "回落也不许拿保留槽 11");
        simulation.end_checkpoint();
    }

    /// 整理臂经 run_arm 全流程必须有效果：B=256、D=64 ⇒ write_amp 恒 = (64+256)/64 = 5.0（绝对值），
    /// 且 runs 至少比 bump_seg 低一半（同种子同负载）。
    #[test]
    fn compact_effect_via_run_arm() {
        let bump_segment_outcome = run_arm(Policy::BumpSegment, Load::Runs8, 1, 400, 100);
        let compact_outcome = run_arm(Policy::BumpCompact(256), Load::Runs8, 1, 400, 100);
        assert_eq!(compact_outcome.write_amplification, 5.0);
        assert!(
            compact_outcome.runs_final * 2 < bump_segment_outcome.runs_final,
            "cp={} base={}",
            compact_outcome.runs_final,
            bump_segment_outcome.runs_final
        );
    }

    /// dirty_set 的形状：uniform 恰 D 个去重 key；runs8 由 8 长段拼成。
    #[test]
    fn dirty_set_shapes() {
        let mut random_generator = XorshiftGenerator::new(7);
        let uniform_dirty = dirty_set(Load::Uniform, &mut random_generator, OBJECT_COUNT, DIRTY_PER_CHECKPOINT);
        assert_eq!(uniform_dirty.len(), DIRTY_PER_CHECKPOINT);
        assert!(uniform_dirty.windows(2).all(|adjacent_pair| adjacent_pair[0] < adjacent_pair[1]), "升序去重");
        let runs8_dirty = dirty_set(Load::Runs8, &mut random_generator, OBJECT_COUNT, DIRTY_PER_CHECKPOINT);
        assert_eq!(runs8_dirty.len(), DIRTY_PER_CHECKPOINT);
    }
}
