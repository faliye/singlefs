//! E151：用户数据落点的到达序与容器臂 —— D3（空间分配）已定项 8 与 D14 已定项 2 欠的基线臂（C239 第 ② 笔）。
//!
//! ## 模型（判据与失败条款的权威登记在 research/prompts/e151-preregistration.md，跑前写死 2026-09-13）
//!
//! 沿用 E93（老化下的放置与碎片度）的槽位 / defer / runs 口径，节点骨架照 E127（分裂合并之下组身份要不要存）但不分裂不合并：
//! 单对象节点装 1 个 key，容器节点装 ≤ 7 个连续 key（D27 已定项 6 的打包，cap 7）。
//! runs 按 key 序数：key k 与 k+1 相邻 ⟺ 同一个节点，或两个节点住的槽相邻，或两个节点是同一个容器的两片（同一个槽）。
//!
//! | 臂 | 放置 | 来源 |
//! |---|---|---|
//! | first_fit（对照） | 最低空槽优先 | E93 |
//! | bump_seg（乙） | bump 段、批内 key 升序 | E93 |
//! | arrival（丙） | bump 段、批内到达序（同一随机数发生器再洗一次牌） | 新 |
//! | container_k16 / k64 | 乙 + 打包（迟滞 16 / 64，每 checkpoint 巡回 ≤ 64 个 key） | 新 |
//! | bump_nb_r1 / r2、bump_cp_b32 / b128 / b256（对照） | E93 原样 | E93 |
//!
//! 提交内生块量 M ∈ {0, 4, 9}：每 checkpoint 从 bump 段另分配 M 个占位块（D3 已定项 5：提交内生块必须从聚簇段分配），
//! 下一 checkpoint 收尾释放进 defer；它们不进 runs。
//! 全空聚簇段数：`free_slots_per_segment == 64` 的段数，增量计数与全扫各算一遍（C243 要的那个量）。
//!
//! ## 跨装置闸（判据 4）
//!
//! 容量 1、不打包、M = 0 时，七条 E93 臂 × 两组负载的 14 个 `aging_median` 必须逐字复现 E93 入库产物
//! `e93-aging-placement-2026-09-03.out`（与 E95 的闸同型，E95 已证过这套放置代码能复现）。

use e7_index_bench::Emitter;
use std::collections::BTreeSet;

const OBJECT_COUNT: usize = 8192; // key 数，与 E93 的对象数同一个几何
const SLOT_COUNT: usize = 10240; // 槽数（填充 80%）
const SLOTS_PER_SEGMENT: usize = 64; // 聚簇段槽数
const DIRTY_PER_CHECKPOINT: usize = 64; // 每 checkpoint 用户脏对象数
const CHECKPOINT_COUNT: u64 = 2000; // checkpoint 数
const SAMPLE_EVERY: u64 = 250;
const SEED_COUNT: u64 = 5;

/// D27 已定项 6：界线 4 KiB、一个容器装 7 个对象（2026-09-13 用户定案）。
const CONTAINER_CAPACITY: usize = 7;
/// 打包巡回：每 checkpoint 最多看 64 个 key（取法，不是条款）。它与 DIRTY_PER_CHECKPOINT 相等这件事没有条款钉着，
/// 第三次跑把它做成参数扫三点（C317 那一例）。
const PACK_SWEEP_PER_CHECKPOINT: usize = 64;
/// 第三次跑扫的巡回预算三点：第一次跑的取法、反推腿副本上见到全空段峰值超过初值的第一点、再放大四倍。
const PACK_SWEEP_SAMPLES: [usize; 3] = [64, 256, 1024];
/// 迟滞两档：对象自上次写以来 ≥ K 个 checkpoint 才打包。
const PACK_HYSTERESIS_SAMPLES: [u64; 2] = [16, 64];
/// 提交内生块量三档：E93 原样 0、E148 第一个事务规模 4、池规模 9。
const METADATA_BLOCKS_SAMPLES: [usize; 3] = [0, 4, 9];
/// 判据 6(a) 的阈值：丙与乙的 runs 中位数之差在 ±5% 以内算打平。
const ARRIVAL_TIE_PERCENT: u64 = 5;
/// 目录 = 32 个连续 key（E122 的每目录文件数）；按目录聚的提示臂与目录局部性指标都按它分组（第四次跑）。
const OBJECTS_PER_DIRECTORY: usize = 32;
/// 「只住一个目录」的块的两种粒度：E122 用的 32 槽，与本装置的 64 槽段。
const DIRECTORY_CHUNK_SLOTS: [usize; 2] = [32, 64];
/// 判据 H1：提示臂的目录遍历段数要 ≤ 减数臂的这个百分比才算买到局部性。
const DIRECTORY_HINT_RUNS_RATIO_PERCENT: u64 = 50;
/// 第五次跑扫的「家固定的提示拿几个尾部段当溢出段」：32 是第四次跑的取法（尾部全部），其余留给提交内生块。
const HOME_OVERFLOW_SEGMENT_SAMPLES: [usize; 4] = [4, 8, 16, 32];

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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    FirstFit,
    BumpSegment,
    /// 丙：批内按均匀置换的到达序（第一次跑的读法）。
    Arrival,
    /// 丙′：按请求到达——D16 已定项 5 的事务切分纪律把一次请求切成若干事务、请求内的单元连着到达；
    /// 请求 = 脏集合里 key 连续的一段，请求之间洗牌、请求内保持 key 序（第二次跑，反推腿打中之后加）。
    ArrivalByRequest,
    Container { hysteresis: u64 },
    /// 容器臂的打包前落点换成最低空槽（第二次跑，反推腿打中之后加）：打包规则一字不改。
    ContainerOnFirstFit { hysteresis: u64 },
    /// 丙（D14 已定项 2 的「按目录」提示，第四次跑加）：重写落到该目录成员所在段里槽号最小的空槽，没有就回落到最低空槽。
    DirectoryHint,
    /// 丙′：目录的家固定为初始所在段 + 一个溢出段，重写落到家里槽号最小的空槽，家满才回落，回落不改变家。
    DirectoryHome,
    BumpNeighbor(usize),
    BumpCompact(usize),
}

impl Arm {
    fn tag(self) -> String {
        match self {
            Arm::FirstFit => "first_fit".into(),
            Arm::BumpSegment => "bump_seg".into(),
            Arm::Arrival => "arrival".into(),
            Arm::ArrivalByRequest => "arrival_by_request".into(),
            Arm::Container { hysteresis } => format!("container_k{hysteresis}"),
            Arm::ContainerOnFirstFit { hysteresis } => format!("container_firstfit_k{hysteresis}"),
            Arm::DirectoryHint => "directory_hint".into(),
            Arm::DirectoryHome => "directory_home".into(),
            Arm::BumpNeighbor(radius) => format!("bump_nb_r{radius}"),
            Arm::BumpCompact(compact_budget) => format!("bump_cp_b{compact_budget}"),
        }
    }
    fn is_e93_arm(self) -> bool {
        match self {
            Arm::FirstFit | Arm::BumpSegment | Arm::BumpNeighbor(_) | Arm::BumpCompact(_) => true,
            Arm::Arrival | Arm::ArrivalByRequest | Arm::Container { .. } | Arm::ContainerOnFirstFit { .. } | Arm::DirectoryHint | Arm::DirectoryHome => false,
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

/// 一个节点：连续的一段 key 住一个槽。单对象节点 1 个 key；容器节点 ≤ 7 个 key；
/// 容器里某个 key 被写脏搬走后，容器留下的两片仍住同一个槽（`shared_slot` 让 runs 把它们当同一个容器）。
struct Sim {
    slot_count: usize,
    slots_per_segment: usize,
    /// 位置 → 节点身份（按首 key 序）。
    order: Vec<u32>,
    node_keys: Vec<Vec<u32>>,
    node_slot: Vec<u32>,
    node_alive: Vec<bool>,
    /// 每个槽上住着几个节点（容器的片可以是 2 个）。
    slot_occupancy: Vec<u8>,
    free: BTreeSet<u32>,
    free_slots_per_segment: Vec<u16>,
    empty_segments: usize,
    open_segment: Option<usize>,
    bump: usize,
    broken: u64,
    deferred: Vec<u32>,
    last_write: Vec<u64>,
    user_writes: u64,
    total_writes: u64,
    container_writes: u64,
    fallback_allocations: u64,
    /// 其中提交内生块的回落次数（绝对值：它们本该恒进聚簇段，D3 已定项 5）。
    metadata_fallback_allocations: u64,
    /// 按目录聚的提示臂：成员所在段里没有空槽、回落到全池最低空槽的次数。
    hint_fallback_allocations: u64,
    sweep: usize,
    pack_cursor: usize,
    metadata_current: Vec<u32>,
    /// 每 checkpoint 打包巡回最多看几个 key；run_arm 用 PACK_SWEEP_PER_CHECKPOINT，第三次跑的扫描按点改它。
    pack_sweep_per_checkpoint: usize,
    /// 家固定的提示拿几个尾部段当溢出段（None = 尾部全部，第四次跑的取法；第五次跑扫 4 / 8 / 16 / 32）。
    home_overflow_segments: Option<usize>,
}

impl Sim {
    /// 初始布局：key k 住槽 k（顺序创建的形态），与 E93 逐字同一套；每个 key 自成一个节点。
    fn new(object_count: usize, slot_count: usize, slots_per_segment: usize) -> Sim {
        assert!(object_count < slot_count && slot_count % slots_per_segment == 0);
        let mut slot_occupancy = vec![0u8; slot_count];
        for slot in 0..object_count {
            slot_occupancy[slot] = 1;
        }
        let free: BTreeSet<u32> = (object_count as u32..slot_count as u32).collect();
        let mut free_slots_per_segment = vec![0u16; slot_count / slots_per_segment];
        for slot in object_count..slot_count {
            free_slots_per_segment[slot / slots_per_segment] += 1;
        }
        let empty_segments = free_slots_per_segment.iter().filter(|count| **count as usize == slots_per_segment).count();
        Sim {
            slot_count,
            slots_per_segment,
            order: (0..object_count as u32).collect(),
            node_keys: (0..object_count as u32).map(|key| vec![key]).collect(),
            node_slot: (0..object_count as u32).collect(),
            node_alive: vec![true; object_count],
            slot_occupancy,
            free,
            free_slots_per_segment,
            empty_segments,
            open_segment: None,
            bump: 0,
            broken: 0,
            deferred: Vec::new(),
            last_write: vec![0; object_count],
            user_writes: 0,
            total_writes: 0,
            container_writes: 0,
            fallback_allocations: 0,
            metadata_fallback_allocations: 0,
            hint_fallback_allocations: 0,
            sweep: 0,
            pack_cursor: 0,
            metadata_current: Vec::new(),
            pack_sweep_per_checkpoint: PACK_SWEEP_PER_CHECKPOINT,
            home_overflow_segments: None,
        }
    }

    fn node_count(&self) -> usize {
        self.order.len()
    }
    fn identifier_at(&self, position: usize) -> u32 {
        self.order[position]
    }
    fn slot_at(&self, position: usize) -> u32 {
        self.node_slot[self.order[position] as usize]
    }
    fn first_key_at(&self, position: usize) -> u32 {
        self.node_keys[self.order[position] as usize][0]
    }
    fn key_count_at(&self, position: usize) -> usize {
        self.node_keys[self.order[position] as usize].len()
    }

    /// 相邻两个节点之间断不断：槽相邻不断。同一个容器的两片之间永远隔着被搬出的那个 key（模型不删 key），
    /// 所以两片在 key 序上不相邻，「同槽不断」那一支写了也走不到——变异实测等价，删掉（单测 `fragments_of_one_container_are_never_adjacent` 留档）。
    fn pair_broken(&self, position: usize) -> bool {
        let left = self.slot_at(position - 1);
        let right = self.slot_at(position);
        right != left + 1
    }
    fn broken_pairs_among(&self, positions: &[usize]) -> u64 {
        positions.iter().filter(|&&position| position >= 1 && position < self.node_count() && self.pair_broken(position)).count() as u64
    }
    fn runs_incremental(&self) -> u64 {
        self.broken + 1
    }
    /// 审计：按 key 序全扫；与增量维护不共享累加代码。
    fn audit_runs(&self) -> u64 {
        let mut runs = 1u64;
        let mut previous_slot: Option<u32> = None;
        let mut previous_node: Option<u32> = None;
        for position in 0..self.node_count() {
            let identifier = self.identifier_at(position);
            let slot = self.node_slot[identifier as usize];
            for _key in &self.node_keys[identifier as usize] {
                if let (Some(previous_slot), Some(previous_node)) = (previous_slot, previous_node) {
                    if previous_node != identifier && slot != previous_slot + 1 {
                        runs += 1;
                    }
                }
                previous_slot = Some(slot);
                previous_node = Some(identifier);
            }
        }
        runs
    }
    fn audit_empty_segments(&self) -> usize {
        assert_eq!(self.slot_occupancy.len(), self.slot_count, "槽表长度就是槽数");
        self.free_slots_per_segment.iter().filter(|count| **count as usize == self.slots_per_segment).count()
    }
    fn keys_held(&self) -> usize {
        self.order.iter().map(|&identifier| self.node_keys[identifier as usize].len()).sum()
    }

    /// 首 key ≤ key 的最后一个节点的位置。
    fn position_holding(&self, key: u32) -> usize {
        let mut low = 0usize;
        let mut high = self.node_count();
        while low < high {
            let middle = (low + high) / 2;
            if self.first_key_at(middle) <= key {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        low.saturating_sub(1)
    }

    // ---- 段计数（判据 2 的全空聚簇段数走这两个入口）----
    fn segment_free_decrement(&mut self, slot: u32) {
        let segment_index = slot as usize / self.slots_per_segment;
        if self.free_slots_per_segment[segment_index] as usize == self.slots_per_segment {
            self.empty_segments -= 1;
        }
        self.free_slots_per_segment[segment_index] -= 1;
    }
    fn segment_free_increment(&mut self, slot: u32) {
        let segment_index = slot as usize / self.slots_per_segment;
        self.free_slots_per_segment[segment_index] += 1;
        if self.free_slots_per_segment[segment_index] as usize == self.slots_per_segment {
            self.empty_segments += 1;
        }
    }

    /// 节点搬到新槽（COW 重写）：旧槽的占用减一、减到 0 才进 defer。
    fn move_node(&mut self, position: usize, new_slot: u32) {
        let before = self.broken_pairs_among(&[position, position + 1]);
        let identifier = self.identifier_at(position);
        let old = self.node_slot[identifier as usize];
        self.release_occupancy(old);
        self.node_slot[identifier as usize] = new_slot;
        self.slot_occupancy[new_slot as usize] += 1;
        let after = self.broken_pairs_among(&[position, position + 1]);
        self.broken = self.broken + after - before;
        self.total_writes += 1;
    }
    fn release_occupancy(&mut self, slot: u32) {
        self.slot_occupancy[slot as usize] -= 1;
        if self.slot_occupancy[slot as usize] == 0 {
            self.deferred.push(slot);
        }
    }

    fn insert_node_at(&mut self, position: usize, identifier: u32) {
        let before = self.broken_pairs_among(&[position]);
        self.order.insert(position, identifier);
        let after = self.broken_pairs_among(&[position, position + 1]);
        self.broken = self.broken + after - before;
    }
    fn remove_node_at(&mut self, position: usize) -> u32 {
        let before = self.broken_pairs_among(&[position, position + 1]);
        let identifier = self.order.remove(position);
        let after = self.broken_pairs_among(&[position]);
        self.broken = self.broken + after - before;
        identifier
    }
    fn new_node(&mut self, keys: Vec<u32>, slot: u32) -> u32 {
        let identifier = self.node_keys.len() as u32;
        self.node_keys.push(keys);
        self.node_slot.push(slot);
        self.node_alive.push(true);
        self.slot_occupancy[slot as usize] += 1;
        identifier
    }

    /// 把一个 key 从它的容器片里搬出来成为单对象节点、写到 `new_slot`：容器片按 key 切成左右两片（同槽）。
    fn extract_key_to_slot(&mut self, key: u32, new_slot: u32) {
        let position = self.position_holding(key);
        let identifier = self.identifier_at(position) as usize;
        let keys = self.node_keys[identifier].clone();
        let index = keys.iter().position(|&existing| existing == key).expect("首 key ≤ key 的节点必须装着它");
        if keys.len() == 1 {
            self.move_node(position, new_slot);
            return;
        }
        let container_slot = self.node_slot[identifier];
        let left: Vec<u32> = keys[..index].to_vec();
        let right: Vec<u32> = keys[index + 1..].to_vec();
        // 原节点留住左片（左片空时留住右片）；单对象节点插在旁边；再多出一片就另开一个同槽的节点。
        // 容器的槽占用从不经过 0，所以不会被误推进 defer。
        let single = self.new_node(vec![key], new_slot);
        if left.is_empty() {
            self.node_keys[identifier] = right;
            self.insert_node_at(position, single);
        } else {
            self.node_keys[identifier] = left;
            self.insert_node_at(position + 1, single);
            if !right.is_empty() {
                let right_identifier = self.new_node(right, container_slot);
                self.insert_node_at(position + 2, right_identifier);
            }
        }
        self.total_writes += 1;
    }

    /// 打包：把 key 序上连续的 `CONTAINER_CAPACITY` 个单对象节点合成一个容器节点写到 `slot`。
    fn pack_into_container(&mut self, first_position: usize, slot: u32) {
        let mut keys: Vec<u32> = Vec::with_capacity(CONTAINER_CAPACITY);
        for _ in 0..CONTAINER_CAPACITY {
            let identifier = self.remove_node_at(first_position) as usize;
            assert_eq!(self.node_keys[identifier].len(), 1, "只打包单对象节点");
            keys.push(self.node_keys[identifier][0]);
            let old_slot = self.node_slot[identifier];
            self.release_occupancy(old_slot);
            self.node_alive[identifier] = false;
            self.node_keys[identifier] = Vec::new();
        }
        let container = self.new_node(keys, slot);
        self.insert_node_at(first_position, container);
        self.total_writes += 1;
        self.container_writes += 1;
    }

    // ---- 放置面：与 E93 / E95 逐字同一套，「key」换成「位置」 ----
    fn allocate_first_fit(&mut self) -> u32 {
        let slot = *self
            .free
            .iter()
            .find(|&&free_slot| self.open_segment != Some(free_slot as usize / self.slots_per_segment))
            .expect("槽用尽：配置违反 D ≤ S−L");
        self.free.remove(&slot);
        self.segment_free_decrement(slot);
        slot
    }
    /// key 现在住哪个槽（容器成员给容器的槽）。
    fn slot_of_key(&self, key: u32) -> u32 {
        let position = self.position_holding(key);
        self.node_slot[self.order[position] as usize]
    }

    /// 按目录聚的提示（D14 已定项 2 臂丙）：候选槽 = 该目录成员现在住的那些段里的空槽，取槽号最小的；
    /// 没有就回落到减数臂（全池最低空槽），回落单独计。目录 = key / OBJECTS_PER_DIRECTORY。
    fn allocate_directory_hint(&mut self, key: u32) -> u32 {
        let directory = key as usize / OBJECTS_PER_DIRECTORY;
        let first_member = (directory * OBJECTS_PER_DIRECTORY) as u32;
        let last_member = (((directory + 1) * OBJECTS_PER_DIRECTORY).min(self.last_write.len())) as u32;
        let mut member_segments: BTreeSet<usize> = BTreeSet::new();
        for member in first_member..last_member {
            member_segments.insert(self.slot_of_key(member) as usize / self.slots_per_segment);
        }
        for segment_index in member_segments {
            if self.open_segment == Some(segment_index) {
                continue;
            }
            let segment_start = (segment_index * self.slots_per_segment) as u32;
            let segment_end = segment_start + self.slots_per_segment as u32;
            if let Some(&slot) = self.free.range(segment_start..segment_end).next() {
                self.free.remove(&slot);
                self.segment_free_decrement(slot);
                return slot;
            }
        }
        self.hint_fallback_allocations += 1;
        self.allocate_first_fit()
    }

    /// 目录的家（D14 已定项 2 臂丙′）：初始所在段，加一个溢出段——初始全空的尾部段按顺序分给各家段，
    /// 尾部有 (S − L) / G 段、家段有 L / G 段，每个溢出段归 (L / G) / ((S − L) / G) 个家段。
    fn home_segments_of_directory(&self, directory: usize) -> [usize; 2] {
        let object_count = self.last_write.len();
        let home_segment = directory * OBJECTS_PER_DIRECTORY / self.slots_per_segment;
        let home_segment_count = object_count / self.slots_per_segment;
        let tail_segment_count = (self.slot_count - object_count) / self.slots_per_segment;
        let overflow_segment_count = self.home_overflow_segments.unwrap_or(tail_segment_count).min(tail_segment_count).max(1);
        let overflow_segment = home_segment_count + home_segment * overflow_segment_count / home_segment_count;
        [home_segment, overflow_segment]
    }

    fn allocate_directory_home(&mut self, key: u32) -> u32 {
        let directory = key as usize / OBJECTS_PER_DIRECTORY;
        for segment_index in self.home_segments_of_directory(directory) {
            if self.open_segment == Some(segment_index) {
                continue;
            }
            let segment_start = (segment_index * self.slots_per_segment) as u32;
            let segment_end = segment_start + self.slots_per_segment as u32;
            if let Some(&slot) = self.free.range(segment_start..segment_end).next() {
                self.free.remove(&slot);
                self.segment_free_decrement(slot);
                return slot;
            }
        }
        self.hint_fallback_allocations += 1;
        self.allocate_first_fit()
    }

    /// 目录局部性（期末量）：每个目录成员按槽排序后的连续段数之和、成员散在几个段里（均值 ×100 与最大值）、
    /// 只住一个目录的 32 槽 / 64 槽块数。
    fn directory_metrics(&self) -> DirectoryMetrics {
        let object_count = self.last_write.len();
        let directory_count = object_count.div_ceil(OBJECTS_PER_DIRECTORY);
        let mut directory_runs_total = 0u64;
        let mut segments_touched_sum = 0usize;
        let mut segments_touched_maximum = 0usize;
        let mut slot_directory: Vec<Option<usize>> = vec![None; self.slot_count];
        for directory in 0..directory_count {
            let first_member = directory * OBJECTS_PER_DIRECTORY;
            let last_member = ((directory + 1) * OBJECTS_PER_DIRECTORY).min(object_count);
            let mut member_slots: Vec<u32> = (first_member..last_member).map(|member| self.slot_of_key(member as u32)).collect();
            for &slot in &member_slots {
                slot_directory[slot as usize] = Some(directory);
            }
            member_slots.sort_unstable();
            member_slots.dedup();
            let mut runs = 0u64;
            let mut previous: Option<u32> = None;
            for &slot in &member_slots {
                if previous.is_none_or(|previous_slot| previous_slot + 1 != slot) {
                    runs += 1;
                }
                previous = Some(slot);
            }
            directory_runs_total += runs;
            let touched: BTreeSet<usize> = member_slots.iter().map(|&slot| slot as usize / self.slots_per_segment).collect();
            segments_touched_sum += touched.len();
            segments_touched_maximum = segments_touched_maximum.max(touched.len());
        }
        let mut single_directory_chunks = [0usize; 2];
        for (chunk_index, chunk_slots) in DIRECTORY_CHUNK_SLOTS.iter().enumerate() {
            for chunk_start in (0..self.slot_count).step_by(*chunk_slots) {
                let mut owner: Option<usize> = None;
                let mut mixed = false;
                for slot in chunk_start..(chunk_start + chunk_slots) {
                    if let Some(directory) = slot_directory[slot] {
                        match owner {
                            None => owner = Some(directory),
                            Some(current) if current != directory => {
                                mixed = true;
                                break;
                            }
                            Some(_) => {}
                        }
                    }
                }
                if owner.is_some() && !mixed {
                    single_directory_chunks[chunk_index] += 1;
                }
            }
        }
        DirectoryMetrics {
            directory_runs_total,
            segments_touched_mean_percent: (segments_touched_sum * 100 / directory_count) as u64,
            segments_touched_maximum,
            single_directory_chunks,
        }
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
                    self.segment_free_decrement(slot);
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
        self.segment_free_decrement(slot);
    }
    fn place_run(&mut self, positions: &[usize]) {
        let slots_per_segment = self.slots_per_segment;
        let owned: Vec<usize> = positions.to_vec();
        for chunk in owned.chunks(slots_per_segment) {
            self.place_chunk(chunk);
        }
    }
    fn place_chunk(&mut self, positions: &[usize]) {
        let chunk_length = positions.len();
        if let Some(segment_index) = self.open_segment {
            if self.slots_per_segment - self.bump >= chunk_length {
                for &position in positions {
                    let slot = (segment_index * self.slots_per_segment + self.bump) as u32;
                    self.bump += 1;
                    self.take_slot(slot);
                    self.move_node(position, slot);
                }
                return;
            }
        }
        if let Some(segment_index) = self.find_empty_segment() {
            self.open_segment = Some(segment_index);
            self.bump = 0;
            for &position in positions {
                let slot = (segment_index * self.slots_per_segment + self.bump) as u32;
                self.bump += 1;
                self.take_slot(slot);
                self.move_node(position, slot);
            }
            return;
        }
        if let Some(start) = self.find_free_run(chunk_length) {
            for (offset_in_run, &position) in positions.iter().enumerate() {
                let slot = start + offset_in_run as u32;
                self.take_slot(slot);
                self.move_node(position, slot);
            }
            return;
        }
        for &position in positions {
            self.fallback_allocations += 1;
            let slot = self.allocate_first_fit();
            self.move_node(position, slot);
        }
    }

    /// 提交内生块：上一轮的块在这一轮被新的 COW 节点换代、进 defer（下一轮才可复用）；这一轮再从 bump 段占 M 块。
    fn allocate_metadata_blocks(&mut self, count: usize) {
        let previous = std::mem::take(&mut self.metadata_current);
        for slot in previous {
            self.deferred.push(slot);
        }
        for _ in 0..count {
            let before = self.fallback_allocations;
            let slot = self.allocate_bump();
            self.metadata_fallback_allocations += self.fallback_allocations - before;
            self.metadata_current.push(slot);
        }
    }

    /// checkpoint 收尾：defer 的槽此刻才真正可复用（D16 新规则 2）；上一轮的提交内生块换代释放。
    fn end_checkpoint(&mut self) {
        for slot in std::mem::take(&mut self.deferred) {
            self.free.insert(slot);
            self.segment_free_increment(slot);
        }
    }

    fn occupied_slots(&self) -> usize {
        self.slot_occupancy.iter().filter(|count| **count > 0).count()
    }
}

/// 选负载脏 key（去重、升序）。与 E93 逐字同一套。
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

/// 到达序：对升序脏集合做一次 Fisher–Yates 洗牌（同一个随机数发生器）。
fn arrival_order(dirty: &[usize], rng: &mut Rng) -> Vec<usize> {
    let mut order = dirty.to_vec();
    for index in (1..order.len()).rev() {
        let other = rng.below(index + 1);
        order.swap(index, other);
    }
    order
}

/// 按请求到达：脏集合里 key 连续的一段是一次请求（runs8 负载一次请求 8 个连续 key，uniform 负载每个 key 自成一次请求）；
/// 请求之间用同一个随机数发生器洗牌，请求内保持 key 序（D16 已定项 5：一次请求切成若干事务、单元连着到达）。
fn arrival_order_by_request(dirty: &[usize], rng: &mut Rng) -> Vec<usize> {
    let mut requests: Vec<Vec<usize>> = Vec::new();
    for &key in dirty {
        match requests.last_mut() {
            Some(request) if *request.last().expect("请求非空") + 1 == key => request.push(key),
            _ => requests.push(vec![key]),
        }
    }
    for index in (1..requests.len()).rev() {
        let other = rng.below(index + 1);
        requests.swap(index, other);
    }
    requests.into_iter().flatten().collect()
}

/// E93 的 bump_neighbor：把脏 key 段向两侧各扩 ≤R 个干净邻居。
fn extend_neighbors(dirty: &[usize], radius: usize, object_count: usize) -> Vec<usize> {
    let mut set: BTreeSet<usize> = dirty.iter().copied().collect();
    let mut runs: Vec<(usize, usize)> = Vec::new();
    for &key in dirty {
        match runs.last_mut() {
            Some((_, run_last)) if *run_last + 1 == key => *run_last = key,
            _ => runs.push((key, key)),
        }
    }
    for (run_first, run_last) in runs {
        for key in (run_first.saturating_sub(radius)..run_first).chain(run_last + 1..(run_last + 1 + radius).min(object_count)) {
            set.insert(key);
        }
    }
    set.into_iter().collect()
}

/// 单对象或容器成员的 COW 重写：单对象节点搬槽，容器成员搬出。
fn rewrite_key(sim: &mut Sim, key: usize, slot: u32, checkpoint: u64) {
    sim.extract_key_to_slot(key as u32, slot);
    sim.last_write[key] = checkpoint;
}

/// 打包一轮：沿 key 序巡回 ≤ sim.pack_sweep_per_checkpoint 个 key，连续 7 个「单对象、冷够 K」的 key 装一个容器。
fn pack_step(sim: &mut Sim, checkpoint: u64, hysteresis: u64) {
    let object_count = sim.last_write.len();
    let mut looked = 0usize;
    let mut run_start: Option<u32> = None;
    let mut run_length = 0usize;
    while looked < sim.pack_sweep_per_checkpoint {
        let key = sim.pack_cursor as u32;
        sim.pack_cursor = (sim.pack_cursor + 1) % object_count;
        looked += 1;
        let position = sim.position_holding(key);
        let eligible = sim.key_count_at(position) == 1 && checkpoint.saturating_sub(sim.last_write[key as usize]) >= hysteresis;
        let contiguous = run_start.is_some_and(|start| start + run_length as u32 == key) && key != 0;
        if eligible && (run_length == 0 || contiguous) {
            if run_length == 0 {
                run_start = Some(key);
            }
            run_length += 1;
            if run_length == CONTAINER_CAPACITY {
                let first_position = sim.position_holding(run_start.expect("有起点"));
                let slot = sim.allocate_bump();
                sim.pack_into_container(first_position, slot);
                run_start = None;
                run_length = 0;
            }
        } else if eligible {
            run_start = Some(key);
            run_length = 1;
        } else {
            run_start = None;
            run_length = 0;
        }
    }
}

/// 目录局部性的四个量（第四次跑，D14 已定项 2 的被减数与减数都报）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct DirectoryMetrics {
    directory_runs_total: u64,
    segments_touched_mean_percent: u64,
    segments_touched_maximum: usize,
    single_directory_chunks: [usize; 2],
}

struct Outcome {
    runs_final: u64,
    empty_segments_final: usize,
    /// 初始的 (S − L) / G 个全空段被吃光的那一轮（全空段数第一次落到 0）；没吃光就是 None。
    reservoir_drained_at: Option<u64>,
    /// 吃光之后全空段数的峰值（吃光之前那批是初始布局给的，不是政策喂出来的）。
    empty_segments_peak_after_drain: usize,
    /// 吃光之后全空段数 > 0 的轮数。
    rounds_with_empty_segments_after_drain: u64,
    write_amp: f64,
    fallback_percent: f64,
    metadata_fallback_allocations: u64,
    container_writes: u64,
    hint_fallback_allocations: u64,
    directory: DirectoryMetrics,
}

fn run_arm(arm: Arm, load: Load, seed: u64, metadata_blocks: usize, checkpoint_count: u64, sample_every: u64) -> Outcome {
    run_arm_with_pack_sweep(arm, load, seed, metadata_blocks, checkpoint_count, sample_every, PACK_SWEEP_PER_CHECKPOINT)
}

/// 同 run_arm，打包巡回预算按参数给（第三次跑的扫描用）。
fn run_arm_with_pack_sweep(arm: Arm, load: Load, seed: u64, metadata_blocks: usize, checkpoint_count: u64, sample_every: u64, pack_sweep_per_checkpoint: usize) -> Outcome {
    run_arm_with_knobs(arm, load, seed, metadata_blocks, checkpoint_count, sample_every, pack_sweep_per_checkpoint, None)
}

/// 同 run_arm，家固定的提示拿几个尾部段当溢出段按参数给（第五次跑的扫描用）。
fn run_arm_with_home_overflow(arm: Arm, load: Load, seed: u64, metadata_blocks: usize, checkpoint_count: u64, sample_every: u64, home_overflow_segments: usize) -> Outcome {
    run_arm_with_knobs(arm, load, seed, metadata_blocks, checkpoint_count, sample_every, PACK_SWEEP_PER_CHECKPOINT, Some(home_overflow_segments))
}

fn run_arm_with_knobs(arm: Arm, load: Load, seed: u64, metadata_blocks: usize, checkpoint_count: u64, sample_every: u64, pack_sweep_per_checkpoint: usize, home_overflow_segments: Option<usize>) -> Outcome {
    let mut sim = Sim::new(OBJECT_COUNT, SLOT_COUNT, SLOTS_PER_SEGMENT);
    sim.pack_sweep_per_checkpoint = pack_sweep_per_checkpoint;
    sim.home_overflow_segments = home_overflow_segments;
    let mut rng = Rng::new(seed);
    let mut reservoir_drained_at: Option<u64> = None;
    let mut empty_segments_peak_after_drain = 0usize;
    let mut rounds_with_empty_segments_after_drain = 0u64;
    for checkpoint in 1..=checkpoint_count {
        sim.allocate_metadata_blocks(metadata_blocks);
        let dirty = dirty_set(load, &mut rng, OBJECT_COUNT, DIRTY_PER_CHECKPOINT);
        sim.user_writes += dirty.len() as u64;
        match arm {
            Arm::FirstFit => {
                for &key in &dirty {
                    let slot = sim.allocate_first_fit();
                    rewrite_key(&mut sim, key, slot, checkpoint);
                }
            }
            Arm::BumpSegment | Arm::BumpCompact(_) | Arm::Container { .. } => {
                for &key in &dirty {
                    let slot = sim.allocate_bump();
                    rewrite_key(&mut sim, key, slot, checkpoint);
                }
            }
            Arm::ContainerOnFirstFit { .. } => {
                for &key in &dirty {
                    let slot = sim.allocate_first_fit();
                    rewrite_key(&mut sim, key, slot, checkpoint);
                }
            }
            Arm::Arrival => {
                let order = arrival_order(&dirty, &mut rng);
                for &key in &order {
                    let slot = sim.allocate_bump();
                    rewrite_key(&mut sim, key, slot, checkpoint);
                }
            }
            Arm::ArrivalByRequest => {
                let order = arrival_order_by_request(&dirty, &mut rng);
                for &key in &order {
                    let slot = sim.allocate_bump();
                    rewrite_key(&mut sim, key, slot, checkpoint);
                }
            }
            Arm::DirectoryHint => {
                for &key in &dirty {
                    let slot = sim.allocate_directory_hint(key as u32);
                    rewrite_key(&mut sim, key, slot, checkpoint);
                }
            }
            Arm::DirectoryHome => {
                for &key in &dirty {
                    let slot = sim.allocate_directory_home(key as u32);
                    rewrite_key(&mut sim, key, slot, checkpoint);
                }
            }
            Arm::BumpNeighbor(radius) => {
                let batch = extend_neighbors(&dirty, radius, OBJECT_COUNT);
                let mut run_start = 0;
                while run_start < batch.len() {
                    let mut run_end = run_start + 1;
                    while run_end < batch.len() && batch[run_end] == batch[run_end - 1] + 1 {
                        run_end += 1;
                    }
                    let positions: Vec<usize> = batch[run_start..run_end].to_vec();
                    sim.place_run(&positions);
                    run_start = run_end;
                }
            }
        }
        if let Arm::BumpCompact(compact_budget) = arm {
            let batch_set: BTreeSet<usize> = dirty.iter().copied().collect();
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
        if let Arm::Container { hysteresis } | Arm::ContainerOnFirstFit { hysteresis } = arm {
            pack_step(&mut sim, checkpoint, hysteresis);
        }
        sim.end_checkpoint();
        match reservoir_drained_at {
            None => {
                if sim.empty_segments == 0 {
                    reservoir_drained_at = Some(checkpoint);
                }
            }
            Some(_) => {
                empty_segments_peak_after_drain = empty_segments_peak_after_drain.max(sim.empty_segments);
                if sim.empty_segments > 0 {
                    rounds_with_empty_segments_after_drain += 1;
                }
            }
        }
        if checkpoint % sample_every == 0 {
            assert_eq!(sim.runs_incremental(), sim.audit_runs(), "t={checkpoint} 增量与审计分叉");
            assert_eq!(sim.keys_held(), OBJECT_COUNT, "t={checkpoint} key 总数不守恒");
            assert_eq!(sim.occupied_slots() + sim.free.len() + sim.deferred.len() + sim.metadata_current.len(), SLOT_COUNT, "t={checkpoint} 槽不守恒");
            assert_eq!(sim.empty_segments, sim.audit_empty_segments(), "t={checkpoint} 全空段计数分叉");
        }
    }
    Outcome {
        runs_final: sim.runs_incremental(),
        empty_segments_final: sim.empty_segments,
        reservoir_drained_at,
        empty_segments_peak_after_drain,
        rounds_with_empty_segments_after_drain,
        write_amp: sim.total_writes as f64 / sim.user_writes as f64,
        fallback_percent: 100.0 * sim.fallback_allocations as f64 / (sim.total_writes + (checkpoint_count as usize * metadata_blocks) as u64) as f64,
        metadata_fallback_allocations: sim.metadata_fallback_allocations,
        container_writes: sim.container_writes,
        hint_fallback_allocations: sim.hint_fallback_allocations,
        directory: sim.directory_metrics(),
    }
}

/// 判据 3：测量的阳性对照（容量 1 的形态）。
fn measurement_control(sim: &mut Sim) -> (u64, u64, u64, u64) {
    let node_count = sim.node_count();
    for position in 0..node_count {
        let target = if position < node_count / 2 { 2 * position as u32 } else { (2 * (position - node_count / 2) + 1) as u32 };
        force_to(sim, position, target);
    }
    let scatter = (sim.runs_incremental(), sim.audit_runs());
    for position in 0..node_count {
        force_to(sim, position, position as u32);
    }
    let compact = (sim.runs_incremental(), sim.audit_runs());
    (scatter.0, scatter.1, compact.0, compact.1)
}

fn force_to(sim: &mut Sim, position: usize, target: u32) {
    if sim.slot_at(position) == target {
        return;
    }
    if sim.slot_occupancy[target as usize] > 0 {
        let other_position = (0..sim.node_count()).find(|&candidate| sim.slot_at(candidate) == target).expect("槽的持有者在位置表里");
        let spare = *sim.free.iter().next().expect("强制放置需要至少一个空槽");
        sim.free.remove(&spare);
        sim.segment_free_decrement(spare);
        sim.move_node(other_position, spare);
    }
    if sim.free.remove(&target) {
        sim.segment_free_decrement(target);
    } else {
        let deferred_position = sim.deferred.iter().position(|&deferred_slot| deferred_slot == target).expect("目标槽既不空也不在 defer");
        sim.deferred.swap_remove(deferred_position);
    }
    sim.move_node(position, target);
    sim.end_checkpoint();
}

/// E93 入库产物 `research/results/e93-aging-placement-2026-09-03.out` 里的 14 个 aging_median，逐字钉死（跨装置闸）。
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

fn e93_arm_from_tag(arm_tag: &str) -> Arm {
    match arm_tag {
        "first_fit" => Arm::FirstFit,
        "bump_seg" => Arm::BumpSegment,
        "bump_nb_r1" => Arm::BumpNeighbor(1),
        "bump_nb_r2" => Arm::BumpNeighbor(2),
        "bump_cp_b32" => Arm::BumpCompact(32),
        "bump_cp_b128" => Arm::BumpCompact(128),
        "bump_cp_b256" => Arm::BumpCompact(256),
        other => panic!("跨装置表里有认不出的臂：{other}"),
    }
}
fn load_from_tag(load_tag: &str) -> Load {
    match load_tag {
        "uniform" => Load::Uniform,
        "runs8" => Load::Runs8,
        other => panic!("跨装置表里有认不出的负载：{other}"),
    }
}

fn median_of_five(mut values: Vec<u64>) -> u64 {
    values.sort_unstable();
    values[2]
}

/// 判据 6(a)：丙与乙的 runs 之差是否在 ±5% 以内。
fn within_tie_band(arrival_runs: u64, bump_runs: u64) -> bool {
    let difference = arrival_runs.abs_diff(bump_runs) * 100;
    difference <= bump_runs * ARRIVAL_TIE_PERCENT
}

/// 判据 1 的锚点几何：8 个 key、16 个槽、段 4。
fn anchor_sim() -> Sim {
    Sim::new(8, 16, 4)
}

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config l={OBJECT_COUNT} s={SLOT_COUNT} g={SLOTS_PER_SEGMENT} d={DIRTY_PER_CHECKPOINT} t={CHECKPOINT_COUNT} model=counting file_ops=0 seeds={SEED_COUNT} container_capacity={CONTAINER_CAPACITY} pack_sweep={PACK_SWEEP_PER_CHECKPOINT} hysteresis=16,64 metadata_blocks=0,4,9 tie_percent={ARRIVAL_TIE_PERCENT}"
        ))
    );

    // 判据 1：手算锚点——乙按 key 序、丙按到达序 [5,1,7,3] 放进段 2 的槽 8..11
    let mut bump_anchor = anchor_sim();
    for key in [1u32, 3, 5, 7] {
        let slot = bump_anchor.allocate_bump();
        bump_anchor.extract_key_to_slot(key, slot);
    }
    let mut arrival_anchor = anchor_sim();
    for key in [5u32, 1, 7, 3] {
        let slot = arrival_anchor.allocate_bump();
        arrival_anchor.extract_key_to_slot(key, slot);
    }
    assert_eq!(bump_anchor.runs_incremental(), 8, "乙锚点：0,[8],2,[9],4,[10],6,[11] 每个 key 都与两侧断 ⇒ 8 段");
    assert_eq!(arrival_anchor.runs_incremental(), 8, "丙锚点：1 住 9、3 住 11、5 住 8、7 住 10，同样 8 段");
    assert_eq!(bump_anchor.audit_runs(), 8);
    assert_eq!(arrival_anchor.audit_runs(), 8);
    let bump_slots: Vec<u32> = (0..8).map(|position| bump_anchor.slot_at(position)).collect();
    let arrival_slots: Vec<u32> = (0..8).map(|position| arrival_anchor.slot_at(position)).collect();
    assert_ne!(bump_slots, arrival_slots, "两条臂的落点必须不同，否则丙没有分辨力");
    let mut container_anchor = anchor_sim();
    pack_step(&mut container_anchor, 0, 0);
    assert_eq!(container_anchor.runs_incremental(), 2, "容器锚点：0..6 装进槽 8 的容器、7 留在槽 7 ⇒ 2 段");
    assert_eq!(container_anchor.audit_runs(), 2);
    assert_eq!(container_anchor.container_writes, 1);
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=anchor bump_slots={bump_slots:?} arrival_slots={arrival_slots:?} bump_runs={} arrival_runs={} container_runs={} container_writes={}",
            bump_anchor.runs_incremental(), arrival_anchor.runs_incremental(), container_anchor.runs_incremental(), container_anchor.container_writes
        ))
    );

    let arms = [
        Arm::FirstFit,
        Arm::BumpSegment,
        Arm::Arrival,
        Arm::ArrivalByRequest,
        Arm::Container { hysteresis: PACK_HYSTERESIS_SAMPLES[0] },
        Arm::Container { hysteresis: PACK_HYSTERESIS_SAMPLES[1] },
        Arm::ContainerOnFirstFit { hysteresis: PACK_HYSTERESIS_SAMPLES[0] },
        Arm::ContainerOnFirstFit { hysteresis: PACK_HYSTERESIS_SAMPLES[1] },
        Arm::DirectoryHint,
        Arm::DirectoryHome,
        Arm::BumpNeighbor(1),
        Arm::BumpNeighbor(2),
        Arm::BumpCompact(32),
        Arm::BumpCompact(128),
        Arm::BumpCompact(256),
    ];

    // 判据 3：阳性对照每条臂都过闸
    for &arm in &arms {
        let mut sim = Sim::new(OBJECT_COUNT, SLOT_COUNT, SLOTS_PER_SEGMENT);
        let (scatter_incremental, scatter_audit, compact_incremental, compact_audit) = measurement_control(&mut sim);
        assert_eq!((scatter_incremental, scatter_audit), (OBJECT_COUNT as u64, OBJECT_COUNT as u64), "{:?} 全隔离布局必须报 runs=L", arm);
        assert_eq!((compact_incremental, compact_audit), (1, 1), "{:?} 连续布局必须报 runs=1", arm);
        println!("{}", emitter.emit_raw(&format!("name=control arm={} scatter_runs={scatter_incremental} compact_runs={compact_incremental}", arm.tag())));
    }

    // 判据 4：跨装置闸——M = 0、不打包、容量 1 的七条 E93 臂逐格复现
    for (arm_tag, load_tag, want) in E93_MEDIANS {
        let arm = e93_arm_from_tag(arm_tag);
        let got = median_of_five((0..SEED_COUNT).map(|seed| run_arm(arm, load_from_tag(load_tag), seed, 0, CHECKPOINT_COUNT, SAMPLE_EVERY).runs_final).collect());
        assert_eq!(got, want, "跨装置对照失败：{arm_tag}/{load_tag} 本装置报 {got}，E93 入库产物里是 {want}");
        println!("{}", emitter.emit_raw(&format!("name=xfixture arm={arm_tag} load={load_tag} median={got} e93_stored={want}")));
    }

    // 判据 5 / 6：主格
    let mut grid: Vec<(String, &'static str, usize, u64, u64, f64, u64, u64)> = Vec::new();
    for &arm in &arms {
        for load in [Load::Uniform, Load::Runs8] {
            for metadata_blocks in METADATA_BLOCKS_SAMPLES {
                if arm.is_e93_arm() && matches!(arm, Arm::BumpNeighbor(_) | Arm::BumpCompact(_)) && metadata_blocks != 0 {
                    continue; // 对照臂只跑 M = 0（它们不是被判的对象）
                }
                let mut finals = Vec::new();
                let mut empties = Vec::new();
                let mut peaks = Vec::new();
                let mut write_amplification = 0.0;
                let mut fallback_percent = 0.0;
                let mut container_writes = 0u64;
                let mut metadata_fallback = 0u64;
                let mut rounds_with_empty = 0u64;
                let mut drained_at: Option<u64> = None;
                for seed in 0..SEED_COUNT {
                    let outcome = run_arm(arm, load, seed, metadata_blocks, CHECKPOINT_COUNT, SAMPLE_EVERY);
                    if seed == 0 {
                        write_amplification = outcome.write_amp;
                        fallback_percent = outcome.fallback_percent;
                        container_writes = outcome.container_writes;
                        metadata_fallback = outcome.metadata_fallback_allocations;
                        rounds_with_empty = outcome.rounds_with_empty_segments_after_drain;
                        drained_at = outcome.reservoir_drained_at;
                    }
                    finals.push(outcome.runs_final);
                    empties.push(outcome.empty_segments_final as u64);
                    peaks.push(outcome.empty_segments_peak_after_drain as u64);
                }
                let runs_median = median_of_five(finals);
                let empty_median = median_of_five(empties);
                let peak_median = median_of_five(peaks);
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=grid arm={} load={} metadata_blocks={metadata_blocks} runs_median={runs_median} empty_segments_median={empty_median} empty_peak_after_drain_median={peak_median} drained_at_seed0={} rounds_with_empty_after_drain_seed0={rounds_with_empty} write_amp={write_amplification:.3} fallback_pct={fallback_percent:.1} metadata_fallback_seed0={metadata_fallback} container_writes_seed0={container_writes}",
                        arm.tag(),
                        load.tag(),
                        drained_at.map_or("never".to_string(), |checkpoint| checkpoint.to_string())
                    ))
                );
                grid.push((arm.tag(), load.tag(), metadata_blocks, runs_median, empty_median, fallback_percent, peak_median, metadata_fallback));
            }
        }
    }

    // 判据 6(a)：丙对乙；6(b)：容器对乙
    let cell = |arm_tag: &str, load_tag: &str, metadata_blocks: usize| {
        grid.iter().find(|row| row.0 == arm_tag && row.1 == load_tag && row.2 == metadata_blocks).map(|row| (row.3, row.4, row.5, row.6, row.7)).expect("格必须存在")
    };
    let mut all_tied = true;
    let mut all_tied_by_request = true;
    let mut any_container_empty_segments = false;
    let mut any_container_first_fit_empty_segments = false;
    for load in ["uniform", "runs8"] {
        for metadata_blocks in METADATA_BLOCKS_SAMPLES {
            let (bump_runs, bump_empty, bump_fallback, _, bump_metadata_fallback) = cell("bump_seg", load, metadata_blocks);
            let (first_fit_runs, first_fit_empty, _, first_fit_peak, first_fit_metadata_fallback) = cell("first_fit", load, metadata_blocks);
            let (arrival_runs, arrival_empty, _, _, _) = cell("arrival", load, metadata_blocks);
            let (by_request_runs, _, _, _, _) = cell("arrival_by_request", load, metadata_blocks);
            let tied = within_tie_band(arrival_runs, bump_runs);
            let tied_by_request = within_tie_band(by_request_runs, bump_runs);
            all_tied &= tied;
            all_tied_by_request &= tied_by_request;
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=verdict_arrival load={load} metadata_blocks={metadata_blocks} bump_runs={bump_runs} arrival_runs={arrival_runs} tied_within_5pct={tied} arrival_by_request_runs={by_request_runs} by_request_tied_within_5pct={tied_by_request} first_fit_runs={first_fit_runs} bump_empty_segments={bump_empty} arrival_empty_segments={arrival_empty} first_fit_empty_segments={first_fit_empty} first_fit_empty_peak_after_drain={first_fit_peak} bump_fallback_pct={bump_fallback:.1} bump_metadata_fallback={bump_metadata_fallback} first_fit_metadata_fallback={first_fit_metadata_fallback}"
                ))
            );
            for hysteresis in PACK_HYSTERESIS_SAMPLES {
                let (container_runs, container_empty, _, container_peak, _) = cell(&format!("container_k{hysteresis}"), load, metadata_blocks);
                let (first_fit_container_runs, first_fit_container_empty, _, first_fit_container_peak, _) = cell(&format!("container_firstfit_k{hysteresis}"), load, metadata_blocks);
                any_container_empty_segments |= container_empty > 0;
                any_container_first_fit_empty_segments |= first_fit_container_empty > 0;
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=verdict_container load={load} metadata_blocks={metadata_blocks} hysteresis={hysteresis} bump_runs={bump_runs} container_runs={container_runs} container_empty_segments={container_empty} container_empty_peak_after_drain={container_peak} container_firstfit_runs={first_fit_container_runs} container_firstfit_empty_segments={first_fit_container_empty} container_firstfit_empty_peak_after_drain={first_fit_container_peak} bump_empty_segments={bump_empty}"
                    ))
                );
            }
        }
    }
    println!(
        "{}",
        emitter.emit_raw(&format!("name=answer arrival_ties_bump_in_every_cell={all_tied} arrival_by_request_ties_bump_in_every_cell={all_tied_by_request} any_container_cell_has_empty_segments={any_container_empty_segments} any_container_firstfit_cell_has_empty_segments={any_container_first_fit_empty_segments} criterion=E151_6"))
    );
    // 第三次跑：巡回预算扫描（跑前登记 e151-r3-prereg.md 的 S1 / S2），container_k16 × 两组负载 × M = 9。
    let sweep_metadata_blocks = METADATA_BLOCKS_SAMPLES[2];
    let sweep_hysteresis = PACK_HYSTERESIS_SAMPLES[0];
    let mut sweep_sensitive = true;
    for load in [Load::Uniform, Load::Runs8] {
        let mut peaks_by_sweep = Vec::new();
        let mut runs_by_sweep = Vec::new();
        for pack_sweep in PACK_SWEEP_SAMPLES {
            let mut finals = Vec::new();
            let mut empties = Vec::new();
            let mut peaks = Vec::new();
            let mut write_amplification = 0.0;
            let mut container_writes = 0u64;
            let mut rounds_with_empty = 0u64;
            for seed in 0..SEED_COUNT {
                let outcome = run_arm_with_pack_sweep(Arm::Container { hysteresis: sweep_hysteresis }, load, seed, sweep_metadata_blocks, CHECKPOINT_COUNT, SAMPLE_EVERY, pack_sweep);
                if seed == 0 {
                    write_amplification = outcome.write_amp;
                    container_writes = outcome.container_writes;
                    rounds_with_empty = outcome.rounds_with_empty_segments_after_drain;
                }
                finals.push(outcome.runs_final);
                empties.push(outcome.empty_segments_final as u64);
                peaks.push(outcome.empty_segments_peak_after_drain as u64);
            }
            let runs_median = median_of_five(finals);
            let empty_median = median_of_five(empties);
            let peak_median = median_of_five(peaks);
            peaks_by_sweep.push(peak_median);
            runs_by_sweep.push(runs_median);
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=sweep_scan arm=container_k{sweep_hysteresis} load={} metadata_blocks={sweep_metadata_blocks} pack_sweep={pack_sweep} runs_median={runs_median} empty_segments_median={empty_median} empty_peak_after_drain_median={peak_median} rounds_with_empty_after_drain_seed0={rounds_with_empty} write_amp={write_amplification:.3} container_writes_seed0={container_writes}",
                    load.tag()
                ))
            );
        }
        let peak_doubles = peaks_by_sweep[2] >= 2 * peaks_by_sweep[0];
        let runs_monotone = runs_by_sweep[0] >= runs_by_sweep[1] && runs_by_sweep[1] >= runs_by_sweep[2];
        sweep_sensitive &= peak_doubles && runs_monotone;
        println!(
            "{}",
            emitter.emit_raw(&format!(
                "name=sweep_verdict load={} peak_at_64={} peak_at_1024={} peak_doubles={peak_doubles} runs_at_64={} runs_at_1024={} runs_monotone_nonincreasing={runs_monotone}",
                load.tag(),
                peaks_by_sweep[0],
                peaks_by_sweep[2],
                runs_by_sweep[0],
                runs_by_sweep[2]
            ))
        );
    }
    println!("{}", emitter.emit_raw(&format!("name=sweep_answer verdict_sensitive_to_pack_sweep={sweep_sensitive} criterion=E151_S1")));
    // 第四次跑：目录局部性（跑前登记 e151-r4-prereg.md 的 H1–H3），减数臂 first_fit 对被减数 directory_hint，另报 bump_seg 与 arrival_by_request。
    let directory_arms = [Arm::FirstFit, Arm::BumpSegment, Arm::ArrivalByRequest, Arm::DirectoryHint, Arm::DirectoryHome];
    let directory_metadata_samples = [METADATA_BLOCKS_SAMPLES[0], METADATA_BLOCKS_SAMPLES[2]];
    let mut hint_halves_everywhere = true;
    let mut hint_more_single_chunks_everywhere = true;
    let mut home_halves_everywhere = true;
    let mut home_more_single_chunks_everywhere = true;
    let mut home_stays_in_few_segments_everywhere = true;
    for load in [Load::Uniform, Load::Runs8] {
        for metadata_blocks in directory_metadata_samples {
            let mut by_arm: Vec<(String, u64, u64, u64, u64, u64, u64, u64)> = Vec::new();
            for arm in directory_arms {
                let mut directory_runs = Vec::new();
                let mut touched_means = Vec::new();
                let mut touched_maximumes = Vec::new();
                let mut chunks_32 = Vec::new();
                let mut chunks_64 = Vec::new();
                let mut key_runs = Vec::new();
                let mut hint_fallback = 0u64;
                let mut fallback_percent = 0.0;
                for seed in 0..SEED_COUNT {
                    let outcome = run_arm(arm, load, seed, metadata_blocks, CHECKPOINT_COUNT, SAMPLE_EVERY);
                    if seed == 0 {
                        hint_fallback = outcome.hint_fallback_allocations;
                        fallback_percent = outcome.fallback_percent;
                    }
                    directory_runs.push(outcome.directory.directory_runs_total);
                    touched_means.push(outcome.directory.segments_touched_mean_percent);
                    touched_maximumes.push(outcome.directory.segments_touched_maximum as u64);
                    chunks_32.push(outcome.directory.single_directory_chunks[0] as u64);
                    chunks_64.push(outcome.directory.single_directory_chunks[1] as u64);
                    key_runs.push(outcome.runs_final);
                }
                let row = (
                    arm.tag(),
                    median_of_five(directory_runs),
                    median_of_five(touched_means),
                    median_of_five(touched_maximumes),
                    median_of_five(chunks_32),
                    median_of_five(chunks_64),
                    median_of_five(key_runs),
                    hint_fallback,
                );
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=directory arm={} load={} metadata_blocks={metadata_blocks} directory_runs_total={} segments_touched_mean_percent={} segments_touched_max={} single_directory_chunks_32={} single_directory_chunks_64={} runs_median={} hint_fallback_seed0={} fallback_pct={fallback_percent:.1}",
                        row.0, load.tag(), row.1, row.2, row.3, row.4, row.5, row.6, row.7
                    ))
                );
                by_arm.push(row);
            }
            let first_fit_row = by_arm.iter().find(|row| row.0 == "first_fit").expect("减数臂在");
            for hint_tag in ["directory_hint", "directory_home"] {
                let hint_row = by_arm.iter().find(|row| row.0 == hint_tag).expect("被减数在");
                let halves = hint_row.1 * 100 <= first_fit_row.1 * DIRECTORY_HINT_RUNS_RATIO_PERCENT;
                let more_single_chunks = hint_row.4 > first_fit_row.4;
                let few_segments = hint_row.3 <= 3 && hint_row.2 <= 300 && first_fit_row.3 >= 10;
                let key_runs_tied = within_tie_band(hint_row.6, first_fit_row.6);
                if hint_tag == "directory_hint" {
                    hint_halves_everywhere &= halves;
                    hint_more_single_chunks_everywhere &= more_single_chunks;
                } else {
                    home_halves_everywhere &= halves;
                    home_more_single_chunks_everywhere &= more_single_chunks;
                    home_stays_in_few_segments_everywhere &= few_segments;
                }
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=verdict_directory arm={hint_tag} load={} metadata_blocks={metadata_blocks} first_fit_directory_runs={} hint_directory_runs={} hint_at_most_half={halves} first_fit_segments_touched_max={} hint_segments_touched_max={} hint_segments_touched_mean_percent={} hint_stays_in_few_segments={few_segments} first_fit_single_chunks_32={} hint_single_chunks_32={} hint_more_single_chunks={more_single_chunks} first_fit_runs={} hint_runs={} key_runs_tied_within_5pct={key_runs_tied}",
                        load.tag(), first_fit_row.1, hint_row.1, first_fit_row.3, hint_row.3, hint_row.2, first_fit_row.4, hint_row.4, first_fit_row.6, hint_row.6
                    ))
                );
            }
        }
    }
    println!(
        "{}",
        emitter.emit_raw(&format!("name=directory_answer hint_halves_directory_runs_in_every_cell={hint_halves_everywhere} hint_more_single_chunks_in_every_cell={hint_more_single_chunks_everywhere} home_halves_directory_runs_in_every_cell={home_halves_everywhere} home_more_single_chunks_in_every_cell={home_more_single_chunks_everywhere} home_stays_in_few_segments_in_every_cell={home_stays_in_few_segments_everywhere} criterion=E151_H1_H3"))
    );
    // 第五次跑：家固定的提示拿几个尾部段当溢出段（跑前登记 e151-r5-prereg.md 的 K1 / K2），directory_home × 两组负载 × M = 9。
    let overflow_metadata_blocks = METADATA_BLOCKS_SAMPLES[2];
    let mut uniform_has_clean_overflow = false;
    let mut runs8_has_clean_overflow = false;
    for load in [Load::Uniform, Load::Runs8] {
        for home_overflow_segments in HOME_OVERFLOW_SEGMENT_SAMPLES {
            let mut directory_runs = Vec::new();
            let mut touched_maximumes = Vec::new();
            let mut empties = Vec::new();
            let mut key_runs = Vec::new();
            let mut metadata_fallback = 0u64;
            let mut hint_fallback = 0u64;
            for seed in 0..SEED_COUNT {
                let outcome = run_arm_with_home_overflow(Arm::DirectoryHome, load, seed, overflow_metadata_blocks, CHECKPOINT_COUNT, SAMPLE_EVERY, home_overflow_segments);
                if seed == 0 {
                    metadata_fallback = outcome.metadata_fallback_allocations;
                    hint_fallback = outcome.hint_fallback_allocations;
                }
                directory_runs.push(outcome.directory.directory_runs_total);
                touched_maximumes.push(outcome.directory.segments_touched_maximum as u64);
                empties.push(outcome.empty_segments_final as u64);
                key_runs.push(outcome.runs_final);
            }
            let touched_maximum = median_of_five(touched_maximumes);
            let clean = metadata_fallback == 0 && touched_maximum <= 3;
            match load {
                Load::Uniform => uniform_has_clean_overflow |= clean,
                Load::Runs8 => runs8_has_clean_overflow |= clean,
            }
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=home_overflow arm=directory_home load={} metadata_blocks={overflow_metadata_blocks} home_overflow_segments={home_overflow_segments} directory_runs_total={} segments_touched_max={touched_maximum} empty_segments_median={} runs_median={} metadata_fallback_seed0={metadata_fallback} hint_fallback_seed0={hint_fallback} clean_and_local={clean}",
                    load.tag(),
                    median_of_five(directory_runs),
                    median_of_five(empties),
                    median_of_five(key_runs)
                ))
            );
        }
    }
    println!("{}", emitter.emit_raw(&format!("name=home_overflow_answer uniform_has_overflow_count_with_zero_metadata_fallback_and_local={uniform_has_clean_overflow} runs8_has_such_overflow_count={runs8_has_clean_overflow} criterion=E151_K1")));
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 判据 1：乙与丙在锚点上落点不同（8 个 key、16 个槽、段 4）。
    #[test]
    fn anchor_bump_and_arrival_land_differently() {
        let mut bump = anchor_sim();
        for key in [1u32, 3, 5, 7] {
            let slot = bump.allocate_bump();
            bump.extract_key_to_slot(key, slot);
        }
        let mut arrival = anchor_sim();
        for key in [5u32, 1, 7, 3] {
            let slot = arrival.allocate_bump();
            arrival.extract_key_to_slot(key, slot);
        }
        assert_eq!((0..8).map(|position| bump.slot_at(position)).collect::<Vec<_>>(), vec![0, 8, 2, 9, 4, 10, 6, 11]);
        assert_eq!((0..8).map(|position| arrival.slot_at(position)).collect::<Vec<_>>(), vec![0, 9, 2, 11, 4, 8, 6, 10]);
        assert_eq!(bump.runs_incremental(), 8);
        assert_eq!(arrival.runs_incremental(), 8);
    }

    /// 判据 1：容器锚点——K = 0 时第一次巡回把 0..6 装进一个容器，7 留下。
    #[test]
    fn anchor_container_packs_first_seven_keys() {
        let mut sim = anchor_sim();
        pack_step(&mut sim, 0, 0);
        assert_eq!(sim.node_count(), 2);
        assert_eq!(sim.key_count_at(0), 7);
        assert_eq!(sim.slot_at(0), 8, "容器 bump 落到段 2 的第一个槽");
        assert_eq!(sim.slot_at(1), 7);
        assert_eq!(sim.runs_incremental(), 2);
        assert_eq!(sim.audit_runs(), 2);
        assert_eq!(sim.container_writes, 1);
        assert_eq!(sim.deferred.len(), 7, "七个单对象槽进 defer");
    }

    /// 容器成员被写脏：搬出成单对象节点，容器留下同槽的两片，runs 增量与审计一致，槽占用减到 0 才进 defer。
    #[test]
    fn dirty_member_leaves_container_as_two_fragments() {
        let mut sim = anchor_sim();
        pack_step(&mut sim, 0, 0);
        sim.end_checkpoint();
        let slot = sim.allocate_bump();
        sim.extract_key_to_slot(3, slot);
        assert_eq!(sim.node_count(), 4, "左片 [0,1,2]、单 3、右片 [4,5,6]、单 7");
        assert_eq!(sim.slot_at(0), sim.slot_at(2), "两片同槽");
        assert_eq!(sim.slot_occupancy[8], 2);
        assert_eq!(sim.runs_incremental(), sim.audit_runs());
        assert_eq!(sim.runs_incremental(), 3, "[0,1,2]@8 | 3@9 | [4,5,6]@8 | 7@7：容器槽 8 与单对象 9 相邻不断，9 → 8 断，8 → 7 断");
        assert_eq!(sim.keys_held(), 8);
        // 再把 0 搬走：原节点留住 [1,2]，两片仍同槽、占用仍是 2
        let slot = sim.allocate_bump();
        sim.extract_key_to_slot(0, slot);
        assert_eq!(sim.slot_occupancy[8], 2);
        assert_eq!(sim.node_count(), 5, "0 | [1,2] | 3 | [4,5,6] | 7");
        assert_eq!(sim.keys_held(), 8);
        assert_eq!(sim.runs_incremental(), sim.audit_runs());
    }

    /// 等价变异留档（mutation-sampling.md）：同一个容器的两片在 key 序上永远不相邻——中间总隔着被搬出的 key，
    /// 所以 pair_broken 不需要「同槽不断」那一支；这里把「不相邻」本身钉成断言。
    #[test]
    fn fragments_of_one_container_are_never_adjacent() {
        let mut sim = anchor_sim();
        pack_step(&mut sim, 0, 0);
        sim.end_checkpoint();
        for key in [3u32, 1, 5] {
            let slot = sim.allocate_bump();
            sim.extract_key_to_slot(key, slot);
            for position in 1..sim.node_count() {
                assert_ne!(sim.slot_at(position), sim.slot_at(position - 1), "同槽的两片相邻了（搬出 {key} 之后）");
            }
            assert_eq!(sim.runs_incremental(), sim.audit_runs());
        }
    }

    /// 容器成员全部搬出后槽进 defer，一个都不早不晚。
    #[test]
    fn container_slot_is_deferred_only_when_empty() {
        let mut sim = anchor_sim();
        pack_step(&mut sim, 0, 0);
        sim.end_checkpoint();
        for key in 0u32..7 {
            let slot = sim.allocate_bump();
            sim.extract_key_to_slot(key, slot);
            let expected = if key == 6 { 1 } else { 0 };
            assert_eq!(sim.deferred.iter().filter(|&&slot| slot == 8).count(), expected, "key {key}");
        }
        assert_eq!(sim.slot_occupancy[8], 0);
        assert_eq!(sim.keys_held(), 8);
        assert_eq!(sim.runs_incremental(), sim.audit_runs());
    }

    /// 打包只收连续 7 个「单对象且冷够」的 key：中间夹一个热 key 就不装。
    #[test]
    fn pack_step_requires_seven_contiguous_cold_singles() {
        let mut sim = anchor_sim();
        sim.last_write[3] = 5;
        pack_step(&mut sim, 5, 1);
        assert_eq!(sim.container_writes, 0, "key 3 刚写过，0..6 凑不齐");
        pack_step(&mut sim, 6, 1);
        assert_eq!(sim.container_writes, 1, "冷够之后下一轮巡回装进去");
    }

    /// 判据 2：守恒在三条新臂上成立（短跑）。
    #[test]
    fn conservation_holds_on_new_arms() {
        for arm in [Arm::Arrival, Arm::Container { hysteresis: 16 }, Arm::Container { hysteresis: 64 }] {
            for metadata_blocks in [0usize, 9] {
                let outcome = run_arm(arm, Load::Runs8, 0, metadata_blocks, 250, 25);
                assert!(outcome.runs_final >= 1 && outcome.runs_final <= OBJECT_COUNT as u64, "{}", arm.tag());
            }
        }
    }

    /// 判据 3：阳性对照分得出全隔离与连续。
    #[test]
    fn measurement_control_discriminates() {
        let mut sim = Sim::new(OBJECT_COUNT, SLOT_COUNT, SLOTS_PER_SEGMENT);
        let (scatter_incremental, scatter_audit, compact_incremental, compact_audit) = measurement_control(&mut sim);
        assert_eq!((scatter_incremental, scatter_audit), (OBJECT_COUNT as u64, OBJECT_COUNT as u64));
        assert_eq!((compact_incremental, compact_audit), (1, 1));
    }

    /// 跨装置闸自己要能红。
    #[test]
    fn xfixture_gate_is_not_a_rubber_stamp() {
        let got = median_of_five((0..SEED_COUNT).map(|seed| run_arm(Arm::BumpCompact(256), Load::Runs8, seed, 0, CHECKPOINT_COUNT, SAMPLE_EVERY).runs_final).collect());
        assert_eq!(got, 1339);
        assert_ne!(got, 1340);
    }

    /// **跨装置闸的本体**：14 格逐格复现 E93。
    #[test]
    fn xfixture_reproduces_every_stored_e93_median() {
        for (arm_tag, load_tag, want) in E93_MEDIANS {
            let got = median_of_five((0..SEED_COUNT).map(|seed| run_arm(e93_arm_from_tag(arm_tag), load_from_tag(load_tag), seed, 0, CHECKPOINT_COUNT, SAMPLE_EVERY).runs_final).collect());
            assert_eq!(got, want, "跨装置对照失败：{arm_tag}/{load_tag}");
        }
    }

    /// 判据 5：bump_seg 在 M = 0 时的回落率与 E93 产物同（种子 0 是 93.5%）。
    #[test]
    fn bump_segment_fallback_matches_e93_at_zero_metadata() {
        let outcome = run_arm(Arm::BumpSegment, Load::Uniform, 0, 0, CHECKPOINT_COUNT, SAMPLE_EVERY);
        assert!((outcome.fallback_percent - 93.5).abs() < 0.05, "E93 产物 bump_seg/uniform/seed0 fallback_pct=93.5，本装置 {:.2}", outcome.fallback_percent);
    }

    /// 到达序真的洗了牌：丙与乙同种子的落点不同、runs 不同（敏感取样点：runs8 负载）。
    #[test]
    fn arrival_arm_differs_from_bump_segment() {
        let bump = run_arm(Arm::BumpSegment, Load::Runs8, 0, 0, 300, 50);
        let arrival = run_arm(Arm::Arrival, Load::Runs8, 0, 0, 300, 50);
        assert_ne!(bump.runs_final, arrival.runs_final);
    }

    /// 提交内生块占段：M = 9 时 bump_seg 的回落率与全空段数与 M = 0 不同，且上一轮的块下一轮释放。
    #[test]
    fn metadata_blocks_occupy_segments_and_are_released_next_checkpoint() {
        let mut sim = Sim::new(64, 128, 8);
        sim.allocate_metadata_blocks(3);
        assert_eq!(sim.metadata_current.len(), 3);
        assert_eq!(sim.free.len(), 64 - 3);
        sim.end_checkpoint();
        assert_eq!(sim.free.len(), 61, "这一轮占的块这一轮不回来");
        sim.allocate_metadata_blocks(0);
        assert_eq!(sim.deferred.len(), 3, "下一轮开始时上一轮的块进 defer");
        sim.end_checkpoint();
        assert_eq!(sim.free.len(), 64, "下一轮收尾才回到空闲");
        let zero = run_arm(Arm::BumpSegment, Load::Uniform, 0, 0, 300, 50);
        let nine = run_arm(Arm::BumpSegment, Load::Uniform, 0, 9, 300, 50);
        assert_ne!((zero.fallback_percent * 10.0) as u64, (nine.fallback_percent * 10.0) as u64, "M 不同回落率要变");
    }

    /// 全空段计数：增量与全扫逐点相等，且初始等于 (S − L) / G。
    #[test]
    fn empty_segment_counter_matches_audit() {
        let mut sim = Sim::new(OBJECT_COUNT, SLOT_COUNT, SLOTS_PER_SEGMENT);
        assert_eq!(sim.empty_segments, (SLOT_COUNT - OBJECT_COUNT) / SLOTS_PER_SEGMENT);
        assert_eq!(sim.empty_segments, sim.audit_empty_segments());
        let slot = sim.allocate_bump();
        sim.extract_key_to_slot(0, slot);
        assert_eq!(sim.empty_segments, (SLOT_COUNT - OBJECT_COUNT) / SLOTS_PER_SEGMENT - 1, "开了一个段");
        assert_eq!(sim.empty_segments, sim.audit_empty_segments());
    }

    /// 按请求到达：请求内 key 序不变、请求之间的顺序变了；runs8 负载上它与均匀置换的到达序落点不同。
    #[test]
    fn arrival_by_request_keeps_order_inside_a_request() {
        let mut rng = Rng::new(7);
        let dirty = vec![10usize, 11, 12, 13, 40, 41, 42, 43, 90];
        let order = arrival_order_by_request(&dirty, &mut rng);
        assert_eq!(order.len(), 9);
        let position = |key: usize| order.iter().position(|&candidate| candidate == key).expect("在");
        assert!(position(10) < position(11) && position(11) < position(12) && position(12) < position(13), "请求内保持 key 序");
        assert!(position(40) < position(41) && position(41) < position(42) && position(42) < position(43));
        assert_eq!(order, vec![40, 41, 42, 43, 10, 11, 12, 13, 90], "种子 7 下三个请求洗成 (40..43, 10..13, 90)，请求之间的顺序变了、请求内没变");
        let by_request = run_arm(Arm::ArrivalByRequest, Load::Runs8, 0, 0, 300, 50);
        let by_key = run_arm(Arm::Arrival, Load::Runs8, 0, 0, 300, 50);
        assert_ne!(by_request.runs_final, by_key.runs_final, "两种到达序在 runs8 上要出不同的数");
    }

    /// 提交内生块的回落分开数：用户数据走最低空槽时它们恒进聚簇段（回落 0 次），走 bump 时被用户数据挤出去。
    #[test]
    fn metadata_fallback_is_zero_when_user_data_stays_out_of_bump_segments() {
        let first_fit = run_arm(Arm::FirstFit, Load::Uniform, 0, 9, 500, 50);
        let bump = run_arm(Arm::BumpSegment, Load::Uniform, 0, 9, 500, 50);
        assert_eq!(first_fit.metadata_fallback_allocations, 0);
        assert!(bump.metadata_fallback_allocations > 0, "用户数据进 bump 段之后提交内生块也回落");
    }

    /// 全空段数的轨迹：初始 (S − L) / G = 32 个全空段，bump 每轮开一个（64 个脏对象正好填满一段），第 32 轮末计数为 0，
    /// 之后再也喂不出一个；最低空槽从不吃光（回收的低位槽先被拿走，尾部那批全空段一直在）。
    #[test]
    fn bump_drains_the_initial_empty_segments_at_checkpoint_32_and_never_refills() {
        let bump = run_arm(Arm::BumpSegment, Load::Uniform, 0, 0, 500, 50);
        assert_eq!(bump.reservoir_drained_at, Some(((SLOT_COUNT - OBJECT_COUNT) / SLOTS_PER_SEGMENT) as u64));
        assert!(
            bump.empty_segments_peak_after_drain == 1 && bump.rounds_with_empty_segments_after_drain > 0 && bump.rounds_with_empty_segments_after_drain < 100,
            "吃光之后偶尔有一段整段被改写腾空、下一轮就被 bump 开掉：峰值 {} 轮数 {}",
            bump.empty_segments_peak_after_drain,
            bump.rounds_with_empty_segments_after_drain
        );
        let first_fit = run_arm(Arm::FirstFit, Load::Uniform, 0, 0, 500, 50);
        assert_eq!(first_fit.reservoir_drained_at, None, "最低空槽不吃尾部那批全空段");
    }

    /// 目录局部性的绝对值：初始布局每个目录一段连续 ⇒ 遍历段数 256、只住一个目录的 32 槽块 256、64 槽块 0（一段正好两个目录）。
    #[test]
    fn initial_layout_directory_metrics_are_pinned() {
        let sim = Sim::new(OBJECT_COUNT, SLOT_COUNT, SLOTS_PER_SEGMENT);
        let metrics = sim.directory_metrics();
        assert_eq!(metrics.directory_runs_total, (OBJECT_COUNT / OBJECTS_PER_DIRECTORY) as u64);
        assert_eq!(metrics.directory_runs_total, 256);
        assert_eq!(metrics.segments_touched_mean_percent, 100);
        assert_eq!(metrics.segments_touched_maximum, 1);
        assert_eq!(metrics.single_directory_chunks, [256, 0]);
    }

    /// 手算锚点：目录 0 的成员住段 0；重写 key 3 时段 0 没有空槽（初始全满），回落到最低空槽 8192（段 128）；
    /// 此后再重写 key 5，段 128 里已有成员（key 3），空槽 8193 在那里 ⇒ 提示把它放到 8193 而不是全池别处。
    #[test]
    fn anchor_directory_hint_places_next_rewrite_beside_the_directory_member() {
        let mut sim = Sim::new(OBJECT_COUNT, SLOT_COUNT, SLOTS_PER_SEGMENT);
        let first = sim.allocate_directory_hint(3);
        assert_eq!(first, 8192, "段 0 全满，回落到全池最低空槽");
        assert_eq!(sim.hint_fallback_allocations, 1);
        rewrite_key(&mut sim, 3, first, 1);
        sim.end_checkpoint();
        let second = sim.allocate_directory_hint(5);
        assert_eq!(second, 3, "key 3 的旧槽 3 在下一轮回到空闲集合，它在段 0 里、是成员所在段里最小的空槽");
        rewrite_key(&mut sim, 5, second, 2);
        sim.end_checkpoint();
        let third = sim.allocate_directory_hint(7);
        assert_eq!(third, 5, "同理落回段 0 的槽 5，而不是段 128");
        assert_eq!(sim.hint_fallback_allocations, 1, "后两次都没回落");
        let elsewhere = sim.allocate_directory_hint(4000);
        assert_eq!(elsewhere, 8193, "目录 125 的成员都在段 62、段 62 全满 ⇒ 回落到全池最低空槽 8193");
        assert_eq!(sim.hint_fallback_allocations, 2);
    }

    /// 溢出段数是参数：None 就是尾部全部 32 段（第四次跑），8 时 128 个家段分 8 个溢出段、其余 24 段留给提交内生块。
    #[test]
    fn home_overflow_count_is_a_parameter() {
        let mut sim = Sim::new(OBJECT_COUNT, SLOT_COUNT, SLOTS_PER_SEGMENT);
        assert_eq!(sim.home_segments_of_directory(255), [127, 159]);
        sim.home_overflow_segments = Some(8);
        assert_eq!(sim.home_segments_of_directory(0), [0, 128]);
        assert_eq!(sim.home_segments_of_directory(255), [127, 135], "8 个溢出段时最后一个家段用第 8 个溢出段");
        sim.home_overflow_segments = Some(64);
        assert_eq!(sim.home_segments_of_directory(255), [127, 159], "超过尾部段数按尾部全部算");
        let all = run_arm_with_home_overflow(Arm::DirectoryHome, Load::Uniform, 0, 9, 300, 50, 32);
        let default_run = run_arm(Arm::DirectoryHome, Load::Uniform, 0, 9, 300, 50);
        assert_eq!(all.directory, default_run.directory, "溢出段 32 与不给参数逐字段同数");
        let eight = run_arm_with_home_overflow(Arm::DirectoryHome, Load::Uniform, 0, 9, 300, 50, 8);
        assert_eq!(eight.metadata_fallback_allocations, 0, "留 24 段给提交内生块时它们不回落");
        assert!(all.metadata_fallback_allocations > 0);
        assert!(eight.hint_fallback_allocations > all.hint_fallback_allocations, "溢出段少了，家满回落的次数只多不少");
    }

    /// 家的编号：目录 d 的家段是 d / 2（两目录合住一段），溢出段按 128 个家段 : 32 个尾部段 = 4 : 1 分。
    #[test]
    fn directory_home_segments_are_pinned() {
        let sim = Sim::new(OBJECT_COUNT, SLOT_COUNT, SLOTS_PER_SEGMENT);
        assert_eq!(sim.home_segments_of_directory(0), [0, 128]);
        assert_eq!(sim.home_segments_of_directory(1), [0, 128]);
        assert_eq!(sim.home_segments_of_directory(7), [3, 128]);
        assert_eq!(sim.home_segments_of_directory(8), [4, 129]);
        assert_eq!(sim.home_segments_of_directory(255), [127, 159]);
    }

    /// 老化 300 轮后（uniform、M = 0）：跟着成员走的提示与减数臂几乎一样散（家只增不减），
    /// 家固定的提示把每个目录钉在家段 + 溢出段里；槽相邻性三条都守不住。数字钉成绝对值的方向。
    #[test]
    fn directory_home_pins_directories_to_few_segments_while_member_hint_drifts() {
        let first_fit = run_arm(Arm::FirstFit, Load::Uniform, 0, 0, 300, 50);
        let hint = run_arm(Arm::DirectoryHint, Load::Uniform, 0, 0, 300, 50);
        let home = run_arm(Arm::DirectoryHome, Load::Uniform, 0, 0, 300, 50);
        assert!(first_fit.directory.segments_touched_maximum >= 20, "first_fit 最多散在 {} 段", first_fit.directory.segments_touched_maximum);
        assert!(hint.directory.segments_touched_maximum >= 15, "跟着成员走的提示最多散在 {} 段", hint.directory.segments_touched_maximum);
        assert!(home.directory.segments_touched_maximum <= 3, "家固定的提示最多散在 {} 段", home.directory.segments_touched_maximum);
        assert!(home.directory.segments_touched_mean_percent <= 300);
        assert!(home.directory.directory_runs_total * 10 > first_fit.directory.directory_runs_total * 5, "槽相邻性谁都守不住：home {} 对 first_fit {}", home.directory.directory_runs_total, first_fit.directory.directory_runs_total);
    }

    /// 巡回预算是参数：预算 64 与 run_arm 逐字同数，预算 1024 打包更多（容器写更多、runs 更少）。
    #[test]
    fn pack_sweep_budget_is_a_parameter_and_moves_packing() {
        let default_budget = run_arm(Arm::Container { hysteresis: 16 }, Load::Runs8, 0, 9, 400, 50);
        let same_budget = run_arm_with_pack_sweep(Arm::Container { hysteresis: 16 }, Load::Runs8, 0, 9, 400, 50, PACK_SWEEP_PER_CHECKPOINT);
        assert_eq!(default_budget.runs_final, same_budget.runs_final);
        assert_eq!(default_budget.container_writes, same_budget.container_writes);
        let large_budget = run_arm_with_pack_sweep(Arm::Container { hysteresis: 16 }, Load::Runs8, 0, 9, 400, 50, 1024);
        assert!(large_budget.container_writes > default_budget.container_writes, "预算 1024 打包更多：{} 对 {}", large_budget.container_writes, default_budget.container_writes);
        assert!(large_budget.runs_final < default_budget.runs_final, "预算 1024 runs 更少：{} 对 {}", large_budget.runs_final, default_budget.runs_final);
    }

    /// 判据 6(a) 的带宽钉死：差 5% 算平、5.1% 不算。
    #[test]
    fn tie_band_is_five_percent_inclusive() {
        assert!(within_tie_band(1050, 1000));
        assert!(within_tie_band(950, 1000));
        assert!(!within_tie_band(1051, 1000));
    }
}
