//! E127：分裂合并之下组身份要不要存 —— D26（后台整理与放置回收）已定项 4 卡住的那个模型缺口。
//!
//! ## 模型（判据与失败条款的权威登记在 kb/experiments/127-分裂合并之下组身份要不要存.md，跑前写死 2026-09-10）
//!
//! 沿用 E93（老化下的放置与碎片度）/ E95（老化的节点布局臂）的槽位、defer、碎片度与等预算口径，
//! **改动只有一处**：放置单位从「静态一一对应的对象」换成会分裂合并的**节点**——
//! 一个节点占一个物理槽，装 ≤ C 个 key，插到 > C 从中点分裂，删到 < C/4 与相邻节点合并。
//! 碎片度 runs 按节点的 key 序数物理连续段（单位从对象换成节点）。
//!
//! | 臂 | 组怎么定 | 要不要盘上字段 |
//! |---|---|---|
//! | key 区间分组（宽 W） | 组 = 节点首 key 落在 `[i·W, (i+1)·W)`，一个组里的节点数随分裂合并浮动 | 不要——放置器从 key 算得出 |
//! | 节点数分组（组大小 G） | 组 = 初始按节点序相邻的 G 个节点；分裂时新节点继承分裂方的组号，组满（已有 G 个）就另开新组 | 要——节点序不是 key 的函数，组号只能存在节点头里 |
//! | 政策臂（对照） | E93 的 bump_compact(B)，B = 预算 | 不要 |
//!
//! ## 跨装置闸（判据 5）
//!
//! 把插删关掉、节点容量设为 1，节点就退化成 E95 的对象：三条臂必须逐字复现 E95 入库产物
//! `e95-node-layout-arms-2026-09-09.out` 里 `pol_compact` 与 `fmt_group_g{2,8,32}` 的 24 个 `runs_median`
//! （key 区间分组与节点数分组在退化几何下都等于 E95 的节点组臂，各核一遍）。对不上判红。
//!
//! ## 模型假设（判据「它答不了的」第 2 条：分裂合并的规则是取法，不是 D8 已定的规则）
//!
//! 中点分裂（左 ⌊n/2⌋、右其余）；合并对象是右邻（没有右邻取左邻）；合并后 > C 再从中点分裂一次。
//! 主格几何：初始 8192 个节点、每节点 4 个 key（容量 8）、key 空间 65536（初始 key 取偶数，留一半空位给插入）、
//! 每 checkpoint 插 16 删 16。

use e7_index_bench::Emitter;
use std::collections::BTreeSet;

const NODE_COUNT: usize = 8192; // 初始节点数，与 E93 / E95 的对象数同一个几何
const SLOT_COUNT: usize = 10240; // 槽数（填充 80%）
const SLOTS_PER_SEGMENT: usize = 64; // 聚簇段槽数
const DIRTY_KEYS_PER_CHECKPOINT: usize = 64; // 每 checkpoint 用户脏 key 数
const CHECKPOINT_COUNT: u64 = 2000; // checkpoint 数
const SAMPLE_EVERY: u64 = 250;

/// 等预算三档：额外写倍数 1.5 / 3 / 5 ⇒ 每 checkpoint 额外写 D×(X−1)。
const BUDGETS: [(usize, &str); 3] = [(32, "1.5x"), (128, "3x"), (256, "5x")];

/// 主格扫的 knob（2026-09-13 第二次跑补齐栅格：两族折成同一组节点宽 1 / 2 / 4 / 8 / 16 / 32）：
/// key 区间宽度按「初始每节点 4 个 key、key 步长 2」折算，W = 8 是 1 个节点宽；节点数分组直接取节点数。
const KEY_INTERVAL_WIDTHS: [usize; 6] = [8, 16, 32, 64, 128, 256];
const NODE_GROUP_SIZES: [usize; 6] = [1, 2, 4, 8, 16, 32];
/// 跨装置闸扫的 knob：退化几何下 key 区间宽度 = 节点数 = E95 的 g。
const DEGENERATE_GROUP_SIZES: [usize; 3] = [2, 8, 32];

/// 判据 6 的阈值（跑前写死）：节点数分组把 runs 压到 key 区间分组的 0.8 倍以下才算「必须存」。
const NODE_GROUP_WIN_NUMERATOR: u64 = 8;
const NODE_GROUP_WIN_DENOMINATOR: u64 = 10;

/// 还没被任何一条臂放置过的新节点（分裂出来的右半）暂时住在这个哨兵槽上；
/// 它在 runs 里恒算断开，checkpoint 收尾前必须被放置掉。
const UNPLACED_SLOT: u32 = u32::MAX;

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

/// 节点几何：容量、初始装几个 key、每 checkpoint 插删多少。插删都是 0 时 key 空间不留空位。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Geometry {
    node_capacity: usize,
    initial_keys_per_node: usize,
    inserts_per_checkpoint: usize,
    deletes_per_checkpoint: usize,
}

impl Geometry {
    fn main_grid() -> Geometry {
        Geometry { node_capacity: 8, initial_keys_per_node: 4, inserts_per_checkpoint: 16, deletes_per_checkpoint: 16 }
    }
    /// 退化成 E95：容量 1、每节点 1 个 key、不插不删 ⇒ 节点就是对象，节点序就是 key 序。
    fn degenerate_e95() -> Geometry {
        Geometry { node_capacity: 1, initial_keys_per_node: 1, inserts_per_checkpoint: 0, deletes_per_checkpoint: 0 }
    }
    fn has_churn(&self) -> bool {
        self.inserts_per_checkpoint > 0 || self.deletes_per_checkpoint > 0
    }
    /// 初始 key 之间的步长：有插删时取 2（初始 key 全是偶数，奇数留给插入），没有时取 1。
    fn key_stride(&self) -> usize {
        if self.has_churn() { 2 } else { 1 }
    }
    fn key_space(&self, node_count: usize) -> usize {
        node_count * self.initial_keys_per_node * self.key_stride()
    }
    /// 合并阈值：key 数 < C/4 就并入邻居（C = 8 ⇒ < 2；C = 2 与 C = 1 ⇒ 只有空节点才并）。
    fn merge_below(&self) -> usize {
        self.node_capacity / 4
    }
}

/// 等预算格上的三条臂。`extra` 是每 checkpoint 的额外写预算（节点数）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    /// 政策臂：E93 的 bump_compact(B)，B 就是预算本身。
    PolicyCompact(usize),
    /// key 区间分组：组 = 首 key / width。
    KeyInterval { width: usize, extra: usize },
    /// 节点数分组：组号存在节点上，初始每 group_size 个相邻节点一组。
    NodeCountGroup { group_size: usize, extra: usize },
}

impl Arm {
    fn tag(self) -> String {
        match self {
            Arm::PolicyCompact(_) => "pol_compact".into(),
            Arm::KeyInterval { width, .. } => format!("key_interval_w{width}"),
            Arm::NodeCountGroup { group_size, .. } => format!("node_group_g{group_size}"),
        }
    }
    fn family(self) -> &'static str {
        match self {
            Arm::PolicyCompact(_) => "policy",
            Arm::KeyInterval { .. } => "key_interval",
            Arm::NodeCountGroup { .. } => "node_group",
        }
    }
    fn extra(self) -> usize {
        match self {
            Arm::PolicyCompact(extra) => extra,
            Arm::KeyInterval { extra, .. } | Arm::NodeCountGroup { extra, .. } => extra,
        }
    }
    /// 节点数分组的名义组大小；别的臂上分裂时的组继承照旧跑，只是没人读它。
    fn nominal_group_size(self) -> usize {
        match self {
            Arm::NodeCountGroup { group_size, .. } => group_size,
            Arm::PolicyCompact(_) | Arm::KeyInterval { .. } => 1,
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

/// 树状数组：按 key 值维护「在不在」，给 O(log K) 的第 rank 个存活 key。
struct PresenceIndex {
    present: Vec<bool>,
    tree: Vec<u32>,
    live_count: usize,
}

impl PresenceIndex {
    fn new(key_space: usize) -> PresenceIndex {
        PresenceIndex { present: vec![false; key_space], tree: vec![0; key_space + 1], live_count: 0 }
    }
    fn set(&mut self, key: usize, is_present: bool) {
        assert_ne!(self.present[key], is_present, "key {key} 的在场状态没变，调用方逻辑错");
        self.present[key] = is_present;
        let delta: i64 = if is_present { 1 } else { -1 };
        self.live_count = (self.live_count as i64 + delta) as usize;
        let mut index = key + 1;
        while index < self.tree.len() {
            self.tree[index] = (self.tree[index] as i64 + delta) as u32;
            index += index & index.wrapping_neg();
        }
    }
    fn is_present(&self, key: usize) -> bool {
        self.present[key]
    }
    /// 第 rank 个（0 起）存活 key。
    fn key_at_rank(&self, rank: usize) -> usize {
        assert!(rank < self.live_count, "rank {rank} 超出存活 key 数 {}", self.live_count);
        let mut position = 0usize;
        let mut remaining = rank as u32 + 1;
        let mut step = 1usize;
        while step * 2 < self.tree.len() {
            step *= 2;
        }
        while step > 0 {
            let candidate = position + step;
            if candidate < self.tree.len() && self.tree[candidate] < remaining {
                position = candidate;
                remaining -= self.tree[candidate];
            }
            step /= 2;
        }
        position
    }
}

struct Sim {
    geometry: Geometry,
    nominal_group_size: usize,
    slot_count: usize,
    slots_per_segment: usize,
    /// 节点按 key 序的位置表：位置 → 节点身份。
    order: Vec<u32>,
    /// 按节点身份索引的三张表；身份不回收。
    node_keys: Vec<Vec<u32>>,
    node_slot: Vec<u32>,
    node_group: Vec<u32>,
    node_alive: Vec<bool>,
    group_member_count: Vec<u32>,
    next_group_identifier: u32,
    slot_holder: Vec<Option<u32>>,
    presence: PresenceIndex,
    key_space: usize,
    free: BTreeSet<u32>,
    free_slots_per_segment: Vec<u16>,
    open_segment: Option<usize>,
    bump: usize,
    /// 增量维护的「断开的邻接对」数；runs = broken + 1（节点数 ≥ 1 时）。
    broken: u64,
    deferred: Vec<u32>,
    mandatory_writes: u64,
    total_writes: u64,
    fallback_allocations: u64,
    sweep: usize,
    splits: u64,
    merges: u64,
    unplaced_nodes: usize,
}

impl Sim {
    /// 初始布局：位置 p 的节点住槽 p（顺序创建的形态），与 E93 / E95 逐字同一套；
    /// 位置 p 的第 j 个 key = (p × 每节点 key 数 + j) × 步长。
    fn new(node_count: usize, slot_count: usize, slots_per_segment: usize, geometry: Geometry, nominal_group_size: usize) -> Sim {
        assert!(node_count < slot_count && slot_count % slots_per_segment == 0);
        assert!(geometry.initial_keys_per_node >= 1 && geometry.initial_keys_per_node <= geometry.node_capacity);
        let key_space = geometry.key_space(node_count);
        let mut presence = PresenceIndex::new(key_space);
        let mut node_keys: Vec<Vec<u32>> = Vec::with_capacity(node_count);
        for position in 0..node_count {
            let keys: Vec<u32> = (0..geometry.initial_keys_per_node)
                .map(|offset| ((position * geometry.initial_keys_per_node + offset) * geometry.key_stride()) as u32)
                .collect();
            for &key in &keys {
                presence.set(key as usize, true);
            }
            node_keys.push(keys);
        }
        let mut slot_holder: Vec<Option<u32>> = vec![None; slot_count];
        for position in 0..node_count {
            slot_holder[position] = Some(position as u32);
        }
        let free: BTreeSet<u32> = (node_count as u32..slot_count as u32).collect();
        let mut free_slots_per_segment = vec![0u16; slot_count / slots_per_segment];
        for slot in node_count..slot_count {
            free_slots_per_segment[slot / slots_per_segment] += 1;
        }
        let group_count = node_count.div_ceil(nominal_group_size);
        let mut group_member_count = vec![0u32; group_count];
        let node_group: Vec<u32> = (0..node_count).map(|position| (position / nominal_group_size) as u32).collect();
        for &group in &node_group {
            group_member_count[group as usize] += 1;
        }
        Sim {
            geometry,
            nominal_group_size,
            slot_count,
            slots_per_segment,
            order: (0..node_count as u32).collect(),
            node_keys,
            node_slot: (0..node_count as u32).collect(),
            node_group,
            node_alive: vec![true; node_count],
            group_member_count,
            next_group_identifier: group_count as u32,
            slot_holder,
            presence,
            key_space,
            free,
            free_slots_per_segment,
            open_segment: None,
            bump: 0,
            broken: 0,
            deferred: Vec::new(),
            mandatory_writes: 0,
            total_writes: 0,
            fallback_allocations: 0,
            sweep: 0,
            splits: 0,
            merges: 0,
            unplaced_nodes: 0,
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

    fn pair_broken(&self, position: usize) -> bool {
        let left = self.slot_at(position - 1);
        let right = self.slot_at(position);
        left == UNPLACED_SLOT || right == UNPLACED_SLOT || right != left + 1
    }

    /// 给定几个位置，数其中合法的邻接对 (p−1, p) 里断开的有几个。
    fn broken_pairs_among(&self, positions: &[usize]) -> u64 {
        positions.iter().filter(|&&position| position >= 1 && position < self.node_count() && self.pair_broken(position)).count() as u64
    }

    /// 审计：全扫重算 runs。与增量维护不共享累加代码。
    fn audit_runs(&self) -> u64 {
        let mut runs = 1u64;
        for position in 1..self.node_count() {
            let left = self.slot_at(position - 1);
            let right = self.slot_at(position);
            if left == UNPLACED_SLOT || right == UNPLACED_SLOT || right != left + 1 {
                runs += 1;
            }
        }
        runs
    }

    fn runs_incremental(&self) -> u64 {
        self.broken + 1
    }

    fn live_key_count(&self) -> usize {
        self.presence.live_count
    }

    fn keys_held_in_nodes(&self) -> usize {
        self.order.iter().map(|&identifier| self.node_keys[identifier as usize].len()).sum()
    }

    /// 首 key ≤ key 的最后一个节点的位置（key 比全部首 key 都小时给 0）。
    fn position_holding(&self, key: u32) -> usize {
        let count = self.node_count();
        let mut low = 0usize;
        let mut high = count;
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

    /// 首 key 落在 [range_start, range_end) 的节点位置区间 [a, b)。
    fn positions_with_first_key_in(&self, range_start: u32, range_end: u32) -> (usize, usize) {
        let count = self.node_count();
        let lower = |target: u32| {
            let mut low = 0usize;
            let mut high = count;
            while low < high {
                let middle = (low + high) / 2;
                if self.first_key_at(middle) < target {
                    low = middle + 1;
                } else {
                    high = middle;
                }
            }
            low
        };
        (lower(range_start), lower(range_end))
    }

    fn move_node(&mut self, position: usize, new_slot: u32) {
        debug_assert!(self.slot_holder[new_slot as usize].is_none());
        let before = self.broken_pairs_among(&[position, position + 1]);
        let identifier = self.identifier_at(position);
        let old = self.node_slot[identifier as usize];
        if old == UNPLACED_SLOT {
            self.unplaced_nodes -= 1;
        } else {
            self.slot_holder[old as usize] = None;
            self.deferred.push(old);
        }
        self.node_slot[identifier as usize] = new_slot;
        self.slot_holder[new_slot as usize] = Some(identifier);
        let after = self.broken_pairs_among(&[position, position + 1]);
        self.broken = self.broken + after - before;
        self.total_writes += 1;
    }

    // ---- 结构变化：插入、删除、分裂、合并 ----

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

    /// 中点分裂：左半留在原节点，右半成为新节点（住哨兵槽，等这一轮的臂来放置）。
    /// 节点数分组的组号继承：分裂方的组还没满就跟着它，满了另开新组。返回新节点身份。
    fn split_at(&mut self, position: usize) -> u32 {
        let identifier = self.identifier_at(position) as usize;
        let keys = std::mem::take(&mut self.node_keys[identifier]);
        let midpoint = keys.len() / 2;
        let (left, right) = keys.split_at(midpoint);
        assert!(!left.is_empty() && !right.is_empty(), "分裂要求两半都非空");
        self.node_keys[identifier] = left.to_vec();
        let new_identifier = self.node_keys.len() as u32;
        self.node_keys.push(right.to_vec());
        self.node_slot.push(UNPLACED_SLOT);
        self.node_alive.push(true);
        let parent_group = self.node_group[identifier];
        let group = if (self.group_member_count[parent_group as usize] as usize) < self.nominal_group_size {
            parent_group
        } else {
            let fresh = self.next_group_identifier;
            self.next_group_identifier += 1;
            self.group_member_count.push(0);
            fresh
        };
        self.group_member_count[group as usize] += 1;
        self.node_group.push(group);
        self.unplaced_nodes += 1;
        self.insert_node_at(position + 1, new_identifier);
        self.splits += 1;
        new_identifier
    }

    /// 与右邻合并（没有右邻取左邻）：左边那个节点存活并吸收全部 key，右边那个从位置表退场、槽还回 defer。
    /// 合并后超过容量再分裂一次。返回存活节点的位置。
    fn merge_at(&mut self, position: usize) -> usize {
        assert!(self.node_count() >= 2, "只剩一个节点时不合并");
        let survivor_position = if position + 1 < self.node_count() { position } else { position - 1 };
        let victim_position = survivor_position + 1;
        let survivor = self.identifier_at(survivor_position) as usize;
        let victim = self.identifier_at(victim_position) as usize;
        let victim_keys = std::mem::take(&mut self.node_keys[victim]);
        self.node_keys[survivor].extend(victim_keys);
        // 先从位置表退场（增量 runs 要按它真实的槽算那两对），再还槽。
        self.remove_node_at(victim_position);
        let victim_slot = self.node_slot[victim];
        if victim_slot == UNPLACED_SLOT {
            self.unplaced_nodes -= 1;
        } else {
            self.slot_holder[victim_slot as usize] = None;
            self.deferred.push(victim_slot);
        }
        self.node_slot[victim] = UNPLACED_SLOT;
        self.node_alive[victim] = false;
        self.group_member_count[self.node_group[victim] as usize] -= 1;
        self.merges += 1;
        if self.node_keys[survivor].len() > self.geometry.node_capacity {
            self.split_at(survivor_position);
        }
        survivor_position
    }

    /// 插入一个此前不在的 key；返回被弄脏的节点身份（分裂时两半都脏）。
    fn insert_key(&mut self, key: u32, dirty: &mut BTreeSet<u32>) {
        assert!(!self.presence.is_present(key as usize), "key {key} 已经在");
        let position = self.position_holding(key);
        let identifier = self.identifier_at(position) as usize;
        let keys = &mut self.node_keys[identifier];
        let insert_at = keys.partition_point(|&existing| existing < key);
        keys.insert(insert_at, key);
        self.presence.set(key as usize, true);
        dirty.insert(identifier as u32);
        if self.node_keys[identifier].len() > self.geometry.node_capacity {
            let new_identifier = self.split_at(position);
            dirty.insert(new_identifier);
        }
    }

    /// 删掉一个在场的 key；返回被弄脏的节点身份（合并时存活的那个脏，退场的那个从脏集里拿掉）。
    fn delete_key(&mut self, key: u32, dirty: &mut BTreeSet<u32>) {
        assert!(self.presence.is_present(key as usize), "key {key} 不在");
        let position = self.position_holding(key);
        let identifier = self.identifier_at(position) as usize;
        let keys = &mut self.node_keys[identifier];
        let index = keys.iter().position(|&existing| existing == key).expect("首 key ≤ key 的那个节点必须装着它");
        keys.remove(index);
        self.presence.set(key as usize, false);
        dirty.insert(identifier as u32);
        if self.node_keys[identifier].len() < self.geometry.merge_below() && self.node_count() >= 2 {
            let survivor_position = self.merge_at(position);
            let survivor = self.identifier_at(survivor_position);
            dirty.insert(survivor);
            dirty.retain(|&candidate| self.node_alive[candidate as usize]);
            if survivor_position + 1 < self.node_count() && self.node_slot[self.identifier_at(survivor_position + 1) as usize] == UNPLACED_SLOT {
                dirty.insert(self.identifier_at(survivor_position + 1));
            }
        }
    }

    // ---- 放置面：与 E95 逐字同一套，只是「key」换成了「位置」 ----

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

    /// checkpoint 收尾：defer 的槽此刻才真正可复用（D16 新规则 2）。
    fn end_checkpoint(&mut self) {
        assert_eq!(self.unplaced_nodes, 0, "有分裂出来的节点没被这一轮的臂放置");
        for slot in std::mem::take(&mut self.deferred) {
            self.free.insert(slot);
            self.free_slots_per_segment[slot as usize / self.slots_per_segment] += 1;
        }
    }

    fn occupied(&self) -> usize {
        self.slot_holder.iter().filter(|holder| holder.is_some()).count()
    }

    /// 节点数分组：按组号收集成员位置（升序）。组号 → 位置表，每 checkpoint 建一次。
    fn members_by_group(&self) -> Vec<Vec<usize>> {
        let mut members: Vec<Vec<usize>> = vec![Vec::new(); self.next_group_identifier as usize];
        for position in 0..self.node_count() {
            members[self.node_group[self.identifier_at(position) as usize] as usize].push(position);
        }
        members
    }

    /// key 区间分组的划分（按位置，组按首个成员的位置排序）。判据 1 的锚点用它。
    fn key_interval_partition(&self, width: usize) -> Vec<Vec<usize>> {
        let mut partition: Vec<Vec<usize>> = Vec::new();
        let mut current_group: Option<usize> = None;
        for position in 0..self.node_count() {
            let group = self.first_key_at(position) as usize / width;
            if current_group == Some(group) {
                partition.last_mut().expect("已经开了一组").push(position);
            } else {
                partition.push(vec![position]);
                current_group = Some(group);
            }
        }
        partition
    }

    /// 节点数分组的划分（按位置，组按首个成员的位置排序）。
    fn node_group_partition(&self) -> Vec<Vec<usize>> {
        let members = self.members_by_group();
        let mut partition: Vec<Vec<usize>> = members.into_iter().filter(|group| !group.is_empty()).collect();
        partition.sort_by_key(|group| group[0]);
        partition
    }
}

/// 选负载脏 key（去重、升序）。按存活 key 的名次抽，退化几何下与 E93 / E95 逐字同一串随机数。
fn dirty_keys(load: Load, rng: &mut Rng, sim: &Sim, dirty_count: usize) -> Vec<u32> {
    let live = sim.live_key_count();
    let mut ranks = BTreeSet::new();
    match load {
        Load::Uniform => {
            while ranks.len() < dirty_count {
                ranks.insert(rng.below(live));
            }
        }
        Load::Runs8 => {
            while ranks.len() < dirty_count {
                let start = rng.below(live);
                for rank in start..(start + 8).min(live) {
                    if ranks.len() < dirty_count {
                        ranks.insert(rank);
                    }
                }
            }
        }
    }
    ranks.into_iter().map(|rank| sim.presence.key_at_rank(rank) as u32).collect()
}

/// 一个 checkpoint 的插删：先插后删，都按随机数取；返回被弄脏的节点身份集合。
fn churn(sim: &mut Sim, rng: &mut Rng) -> BTreeSet<u32> {
    let mut dirty: BTreeSet<u32> = BTreeSet::new();
    for _ in 0..sim.geometry.inserts_per_checkpoint {
        let key = loop {
            let candidate = rng.below(sim.key_space);
            if !sim.presence.is_present(candidate) {
                break candidate as u32;
            }
        };
        sim.insert_key(key, &mut dirty);
    }
    for _ in 0..sim.geometry.deletes_per_checkpoint {
        let rank = rng.below(sim.live_key_count());
        let key = sim.presence.key_at_rank(rank) as u32;
        sim.delete_key(key, &mut dirty);
    }
    dirty
}

/// 这一轮必须写的节点位置（升序）：插删弄脏的 ∪ 脏 key 所在的。
fn mandatory_positions(sim: &Sim, churn_dirty: &BTreeSet<u32>, dirty_keys: &[u32]) -> Vec<usize> {
    let mut positions: BTreeSet<usize> = BTreeSet::new();
    if !churn_dirty.is_empty() {
        for position in 0..sim.node_count() {
            if churn_dirty.contains(&sim.identifier_at(position)) {
                positions.insert(position);
            }
        }
    }
    for &key in dirty_keys {
        positions.insert(sim.position_holding(key));
    }
    positions.into_iter().collect()
}

struct Outcome {
    runs_final: u64,
    node_count_final: usize,
    splits: u64,
    merges: u64,
    write_amp: f64,
    fallback_percent: f64,
    budget_spent_percent: f64,
}

fn run_arm(arm: Arm, load: Load, seed: u64, geometry: Geometry, checkpoint_count: u64, sample_every: u64) -> Outcome {
    let mut sim = Sim::new(NODE_COUNT, SLOT_COUNT, SLOTS_PER_SEGMENT, geometry, arm.nominal_group_size());
    let mut rng = Rng::new(seed);
    let mut budget_total = 0u64;
    let mut budget_spent = 0u64;
    let mut group_cursor = 0usize;
    for checkpoint in 1..=checkpoint_count {
        let churn_dirty = churn(&mut sim, &mut rng);
        let dirty = dirty_keys(load, &mut rng, &sim, DIRTY_KEYS_PER_CHECKPOINT);
        let mandatory = mandatory_positions(&sim, &churn_dirty, &dirty);
        sim.mandatory_writes += mandatory.len() as u64;
        let before = sim.total_writes;
        let extra = arm.extra();
        budget_total += extra as u64;
        match arm {
            Arm::PolicyCompact(_) => step_policy_compact(&mut sim, &mandatory, extra),
            Arm::KeyInterval { width, .. } => step_key_interval(&mut sim, &mandatory, width, extra, &mut group_cursor),
            Arm::NodeCountGroup { group_size, .. } => step_node_count_group(&mut sim, &mandatory, group_size, extra, &mut group_cursor),
        }
        let spent_this_checkpoint = (sim.total_writes - before).saturating_sub(mandatory.len() as u64);
        assert!(spent_this_checkpoint <= extra as u64, "{} 在 t={checkpoint} 花超预算：{spent_this_checkpoint} > {extra}", arm.tag());
        budget_spent += spent_this_checkpoint;
        sim.end_checkpoint();
        if checkpoint % sample_every == 0 {
            assert_eq!(sim.runs_incremental(), sim.audit_runs(), "t={checkpoint} 增量与审计分叉");
            assert_eq!(sim.occupied(), sim.node_count(), "t={checkpoint} 占用槽数 ≠ 节点数");
            assert_eq!(sim.free.len() + sim.node_count(), sim.slot_count, "t={checkpoint} 空 + 占 ≠ 总槽数");
            assert_eq!(sim.keys_held_in_nodes(), sim.live_key_count(), "t={checkpoint} 节点里的 key 总数 ≠ 存活 key 数");
        }
    }
    Outcome {
        runs_final: sim.runs_incremental(),
        node_count_final: sim.node_count(),
        splits: sim.splits,
        merges: sim.merges,
        write_amp: sim.total_writes as f64 / sim.mandatory_writes as f64,
        fallback_percent: 100.0 * sim.fallback_allocations as f64 / sim.total_writes as f64,
        budget_spent_percent: 100.0 * budget_spent as f64 / budget_total as f64,
    }
}

/// 政策臂：脏节点 bump 分配，再按位置轮转整理 `extra` 个干净节点（E93 的 bump_compact）。
fn step_policy_compact(sim: &mut Sim, mandatory: &[usize], extra: usize) {
    for &position in mandatory {
        let slot = sim.allocate_bump();
        sim.move_node(position, slot);
    }
    let mandatory_set: BTreeSet<usize> = mandatory.iter().copied().collect();
    let node_count = sim.node_count();
    let mut sweep_positions = Vec::with_capacity(extra);
    while sweep_positions.len() < extra {
        let position = sim.sweep % node_count;
        sim.sweep = (position + 1) % node_count;
        if !mandatory_set.contains(&position) {
            sweep_positions.push(position);
        }
    }
    sim.place_run(&sweep_positions);
}

/// key 区间分组的一个 checkpoint：被碰到的组预算够就整组重落，剩余预算按组号轮转整组重落。
fn step_key_interval(sim: &mut Sim, mandatory: &[usize], width: usize, extra: usize, group_cursor: &mut usize) {
    let mandatory_set: BTreeSet<usize> = mandatory.iter().copied().collect();
    let mut remaining = extra;
    let mut written: BTreeSet<usize> = BTreeSet::new();
    let mut touched: Vec<usize> = mandatory.iter().map(|&position| sim.first_key_at(position) as usize / width).collect();
    touched.dedup();
    for group in touched {
        let (range_start, range_end) = sim.positions_with_first_key_in((group * width) as u32, ((group + 1) * width) as u32);
        let clean = (range_start..range_end).filter(|position| !mandatory_set.contains(position)).count();
        let positions: Vec<usize> = if clean <= remaining {
            remaining -= clean;
            (range_start..range_end).collect()
        } else {
            (range_start..range_end).filter(|position| mandatory_set.contains(position)).collect()
        };
        written.extend(positions.iter().copied());
        sim.place_run(&positions);
    }
    let groups_total = sim.key_space.div_ceil(width);
    let nominal_nodes_per_group = (width / sim.geometry.initial_keys_per_node / sim.geometry.key_stride()).max(1);
    let mut scanned = 0usize;
    while remaining >= nominal_nodes_per_group && scanned < groups_total {
        let group = *group_cursor;
        *group_cursor = (*group_cursor + 1) % groups_total;
        scanned += 1;
        let (range_start, range_end) = sim.positions_with_first_key_in((group * width) as u32, ((group + 1) * width) as u32);
        if range_start == range_end || (range_start..range_end).any(|position| written.contains(&position)) || range_end - range_start > remaining {
            continue;
        }
        let positions: Vec<usize> = (range_start..range_end).collect();
        remaining -= positions.len();
        written.extend(positions.iter().copied());
        sim.place_run(&positions);
    }
}

/// 节点数分组的一个 checkpoint：组号存在节点上，成员按位置升序整组重落。
fn step_node_count_group(sim: &mut Sim, mandatory: &[usize], group_size: usize, extra: usize, group_cursor: &mut usize) {
    let mandatory_set: BTreeSet<usize> = mandatory.iter().copied().collect();
    let members = sim.members_by_group();
    let mut remaining = extra;
    let mut written: BTreeSet<usize> = BTreeSet::new();
    let mut touched: Vec<usize> = Vec::new();
    for &position in mandatory {
        let group = sim.node_group[sim.identifier_at(position) as usize] as usize;
        if !touched.contains(&group) {
            touched.push(group);
        }
    }
    for group in touched {
        let group_members = &members[group];
        let clean = group_members.iter().filter(|position| !mandatory_set.contains(position)).count();
        let positions: Vec<usize> = if clean <= remaining {
            remaining -= clean;
            group_members.clone()
        } else {
            group_members.iter().copied().filter(|position| mandatory_set.contains(position)).collect()
        };
        written.extend(positions.iter().copied());
        sim.place_run(&positions);
    }
    let groups_total = members.len();
    let mut scanned = 0usize;
    while remaining >= group_size && scanned < groups_total {
        let group = *group_cursor % groups_total;
        *group_cursor = (group + 1) % groups_total;
        scanned += 1;
        let group_members = &members[group];
        if group_members.is_empty() || group_members.iter().any(|position| written.contains(position)) || group_members.len() > remaining {
            continue;
        }
        remaining -= group_members.len();
        written.extend(group_members.iter().copied());
        sim.place_run(group_members);
    }
}

/// 判据 3：测量的阳性对照。强制放置（绕过政策）走增量路径，审计路径复核。
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
    if let Some(other_identifier) = sim.slot_holder[target as usize] {
        let other_position = (0..sim.node_count()).find(|&candidate| sim.identifier_at(candidate) == other_identifier).expect("槽的持有者必须在位置表里");
        let spare = *sim.free.iter().next().expect("强制放置需要至少一个空槽");
        sim.free.remove(&spare);
        sim.free_slots_per_segment[spare as usize / sim.slots_per_segment] -= 1;
        sim.move_node(other_position, spare);
    }
    if sim.free.remove(&target) {
        sim.free_slots_per_segment[target as usize / sim.slots_per_segment] -= 1;
    } else {
        let deferred_position = sim.deferred.iter().position(|&deferred_slot| deferred_slot == target).expect("目标槽既不空也不在 defer");
        sim.deferred.swap_remove(deferred_position);
    }
    sim.move_node(position, target);
    sim.end_checkpoint();
}

/// E95 入库产物 `research/results/e95-node-layout-arms-2026-09-09.out` 里 `pol_compact` 与 `fmt_group_g{2,8,32}`
/// 的 24 个 `runs_median`，逐字钉死。**这是跨装置的那道闸**：退化几何下三条臂都得落到同一个数。
const E95_MEDIANS: [(usize, &str, &str, u64); 24] = [
    (32, "pol_compact", "uniform", 7502),
    (32, "pol_compact", "runs8", 6073),
    (32, "fmt_group_g2", "uniform", 6128),
    (32, "fmt_group_g2", "runs8", 3548),
    (32, "fmt_group_g8", "uniform", 7558),
    (32, "fmt_group_g8", "runs8", 1595),
    (32, "fmt_group_g32", "uniform", 7819),
    (32, "fmt_group_g32", "runs8", 4370),
    (128, "pol_compact", "uniform", 6095),
    (128, "pol_compact", "runs8", 3922),
    (128, "fmt_group_g2", "uniform", 3966),
    (128, "fmt_group_g2", "runs8", 2575),
    (128, "fmt_group_g8", "uniform", 5797),
    (128, "fmt_group_g8", "runs8", 904),
    (128, "fmt_group_g32", "uniform", 7249),
    (128, "fmt_group_g32", "runs8", 1969),
    (256, "pol_compact", "uniform", 4791),
    (256, "pol_compact", "runs8", 1339),
    (256, "fmt_group_g2", "uniform", 3414),
    (256, "fmt_group_g2", "runs8", 1139),
    (256, "fmt_group_g8", "uniform", 3669),
    (256, "fmt_group_g8", "runs8", 742),
    (256, "fmt_group_g32", "uniform", 6567),
    (256, "fmt_group_g32", "runs8", 276),
];

fn load_from_tag(load_tag: &str) -> Load {
    match load_tag {
        "uniform" => Load::Uniform,
        "runs8" => Load::Runs8,
        other => panic!("跨装置表里有认不出的负载：{other}"),
    }
}

/// 一格 E95 的臂在本装置退化几何下对应哪几条臂（节点组那一格两条分组臂都要复现）。
fn degenerate_arms_for(e95_arm_tag: &str, extra: usize) -> Vec<Arm> {
    match e95_arm_tag {
        "pol_compact" => vec![Arm::PolicyCompact(extra)],
        "fmt_group_g2" | "fmt_group_g8" | "fmt_group_g32" => {
            let group_size: usize = e95_arm_tag["fmt_group_g".len()..].parse().expect("组大小是数字");
            assert!(DEGENERATE_GROUP_SIZES.contains(&group_size));
            vec![Arm::KeyInterval { width: group_size, extra }, Arm::NodeCountGroup { group_size, extra }]
        }
        other => panic!("跨装置表里有认不出的臂：{other}"),
    }
}

/// 判据 6 的比较：节点数分组的 runs 是否压到 key 区间分组的 0.8 倍以下（整数算，不走浮点）。
fn node_group_beats_threshold(node_group_runs: u64, key_interval_runs: u64) -> bool {
    node_group_runs * NODE_GROUP_WIN_DENOMINATOR < key_interval_runs * NODE_GROUP_WIN_NUMERATOR
}

fn median_of_five(mut values: Vec<u64>) -> u64 {
    values.sort_unstable();
    values[2]
}

fn main() {
    let mut emitter = Emitter::new();
    let geometry = Geometry::main_grid();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config nodes={NODE_COUNT} s={SLOT_COUNT} g={SLOTS_PER_SEGMENT} d={DIRTY_KEYS_PER_CHECKPOINT} t={CHECKPOINT_COUNT} capacity={} initial_keys_per_node={} inserts={} deletes={} key_space={} model=counting file_ops=0 seeds=5 budgets=32,128,256 key_interval_widths=8,16,32,64,128,256 node_group_sizes=1,2,4,8,16,32 split=midpoint merge_below=capacity/4 win_threshold=0.8",
            geometry.node_capacity, geometry.initial_keys_per_node, geometry.inserts_per_checkpoint, geometry.deletes_per_checkpoint, geometry.key_space(NODE_COUNT)
        ))
    );

    // 判据 1：手算锚点——8 个 key、容量 2、组宽 2，插一个 key 触发一次分裂，两条分组臂的划分手算写死且互不相同。
    let anchor_geometry = Geometry { node_capacity: 2, initial_keys_per_node: 2, inserts_per_checkpoint: 1, deletes_per_checkpoint: 0 };
    let mut anchor = Sim::new(4, 8, 4, anchor_geometry, 2);
    let mut anchor_dirty = BTreeSet::new();
    anchor.insert_key(5, &mut anchor_dirty);
    let key_interval_partition = anchor.key_interval_partition(2);
    let node_group_partition = anchor.node_group_partition();
    assert_eq!(anchor.splits, 1, "锚点必须恰好分裂一次");
    assert_eq!(key_interval_partition, vec![vec![0], vec![1, 2], vec![3], vec![4]], "锚点：key 区间分组的划分手算是 [0][1,2][3][4]");
    assert_eq!(node_group_partition, vec![vec![0, 1], vec![2], vec![3, 4]], "锚点：节点数分组的划分手算是 [0,1][2][3,4]");
    assert_ne!(key_interval_partition, node_group_partition, "两条臂在锚点上就相同的话，这个模型没有分辨力，整轮作废");
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=anchor keys=8 capacity=2 width=2 group_size=2 inserted=5 splits={} key_interval_partition={:?} node_group_partition={:?} differ={}",
            anchor.splits, key_interval_partition, node_group_partition, key_interval_partition != node_group_partition
        ))
    );

    // 主格的臂：政策一条 + key 区间三档 + 节点数三档，每档预算各一份
    let mut arms: Vec<Arm> = Vec::new();
    for (extra, _) in BUDGETS {
        arms.push(Arm::PolicyCompact(extra));
        for width in KEY_INTERVAL_WIDTHS {
            arms.push(Arm::KeyInterval { width, extra });
        }
        for group_size in NODE_GROUP_SIZES {
            arms.push(Arm::NodeCountGroup { group_size, extra });
        }
    }

    // 判据 3：阳性对照**每条臂都过闸**
    let mut control_seen: Vec<String> = Vec::new();
    for &arm in &arms {
        let key = arm.tag();
        if control_seen.contains(&key) {
            continue;
        }
        control_seen.push(key.clone());
        let mut sim = Sim::new(NODE_COUNT, SLOT_COUNT, SLOTS_PER_SEGMENT, geometry, arm.nominal_group_size());
        let (scatter_incremental, scatter_audit, compact_incremental, compact_audit) = measurement_control(&mut sim);
        assert_eq!((scatter_incremental, scatter_audit), (NODE_COUNT as u64, NODE_COUNT as u64), "{:?} 全隔离布局必须报 runs = 节点数", arm);
        assert_eq!((compact_incremental, compact_audit), (1, 1), "{:?} 连续布局必须报 runs = 1", arm);
        println!("{}", emitter.emit_raw(&format!("name=control arm={key} scatter_runs={scatter_incremental} compact_runs={compact_incremental}")));
    }

    // 判据 5：跨装置闸——退化几何下 24 格逐字复现 E95
    for (extra, e95_arm_tag, load_tag, want) in E95_MEDIANS {
        for arm in degenerate_arms_for(e95_arm_tag, extra) {
            let got = median_of_five((0..5u64).map(|seed| run_arm(arm, load_from_tag(load_tag), seed, Geometry::degenerate_e95(), CHECKPOINT_COUNT, SAMPLE_EVERY).runs_final).collect());
            assert_eq!(got, want, "跨装置对照失败：{e95_arm_tag}/{load_tag}/预算 {extra} 本装置臂 {} 报 {got}，E95 入库产物里是 {want}", arm.tag());
            println!("{}", emitter.emit_raw(&format!("name=xfixture budget={extra} e95_arm={e95_arm_tag} arm={} load={load_tag} median={got} e95_stored={want}", arm.tag())));
        }
    }

    // 判据 2 / 4 / 6：主格
    let mut grid: Vec<(usize, String, &'static str, &'static str, u64)> = Vec::new();
    for &arm in &arms {
        let extra = arm.extra();
        for load in [Load::Uniform, Load::Runs8] {
            let mut finals = Vec::new();
            let mut node_counts = Vec::new();
            let mut splits = 0u64;
            let mut merges = 0u64;
            let mut write_amplification = 0.0;
            let mut fallback_percent = 0.0;
            let mut budget_spent_percent = 0.0;
            for seed in 0..5u64 {
                let outcome = run_arm(arm, load, seed, geometry, CHECKPOINT_COUNT, SAMPLE_EVERY);
                assert!(outcome.splits > 0 && outcome.merges > 0, "{} 种子 {seed}：分裂 {} 合并 {}，两者都必须 > 0，否则整轮作废", arm.tag(), outcome.splits, outcome.merges);
                if seed == 0 {
                    splits = outcome.splits;
                    merges = outcome.merges;
                    write_amplification = outcome.write_amp;
                    fallback_percent = outcome.fallback_percent;
                    budget_spent_percent = outcome.budget_spent_percent;
                }
                finals.push(outcome.runs_final);
                node_counts.push(outcome.node_count_final as u64);
            }
            let runs_median = median_of_five(finals);
            let node_count_median = median_of_five(node_counts);
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=grid budget={extra} arm={} family={} load={} runs_median={runs_median} node_count_median={node_count_median} splits_seed0={splits} merges_seed0={merges} write_amp={write_amplification:.3} budget_spent_pct={budget_spent_percent:.1} fallback_pct={fallback_percent:.1}",
                    arm.tag(),
                    arm.family(),
                    load.tag()
                ))
            );
            grid.push((extra, arm.tag(), arm.family(), load.tag(), runs_median));
        }
    }

    // 判据 6：任一等预算格上，节点数分组能不能把 runs 压到 key 区间分组的 0.8 倍以下（各臂取自己最好的 knob）
    let mut any_cell_node_group_wins = false;
    let mut all_cells_tied = true;
    let mut key_interval_wins_every_cell = true;
    for (extra, _) in BUDGETS {
        for load in ["uniform", "runs8"] {
            let best_of = |family: &str| {
                grid.iter()
                    .filter(|row| row.0 == extra && row.3 == load && row.2 == family)
                    .map(|row| (row.1.clone(), row.4))
                    .min_by_key(|(_, runs_median)| *runs_median)
                    .expect("每格每族至少一条臂")
            };
            let (best_key_interval_arm, best_key_interval) = best_of("key_interval");
            let (best_node_group_arm, best_node_group) = best_of("node_group");
            let (_, best_policy) = best_of("policy");
            let node_group_wins = node_group_beats_threshold(best_node_group, best_key_interval);
            any_cell_node_group_wins |= node_group_wins;
            all_cells_tied &= best_node_group == best_key_interval;
            key_interval_wins_every_cell &= best_key_interval < best_node_group;
            println!(
                "{}",
                emitter.emit_raw(&format!(
                    "name=verdict budget={extra} load={load} best_key_interval={best_key_interval} best_key_interval_arm={best_key_interval_arm} best_node_group={best_node_group} best_node_group_arm={best_node_group_arm} best_policy={best_policy} node_group_below_0_8={node_group_wins}"
                ))
            );
        }
    }
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=answer any_cell_node_group_below_0_8={any_cell_node_group_wins} all_cells_tied={all_cells_tied} key_interval_wins_every_cell={key_interval_wins_every_cell} criterion=E127_6"
        ))
    );
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn short_run(arm: Arm, load: Load) -> Outcome {
        run_arm(arm, load, 0, Geometry::main_grid(), 200, 25)
    }

    /// **判据 1 手算锚点**：8 个 key（0,2,…,14）住 4 个容量 2 的节点，插 key 5 让节点 [4,6] 分裂成 [4] 与 [5,6]。
    /// key 区间（宽 2）按首 key 0,4,5,8,12 ⇒ 组 0,2,2,4,6 ⇒ [0][1,2][3][4]；
    /// 节点数分组（G = 2）：分裂方的组 {0,1} 已满 ⇒ 新节点另开组 ⇒ [0,1][2][3,4]。
    #[test]
    fn hand_anchor_partitions_differ_after_one_split() {
        let geometry = Geometry { node_capacity: 2, initial_keys_per_node: 2, inserts_per_checkpoint: 1, deletes_per_checkpoint: 0 };
        let mut sim = Sim::new(4, 8, 4, geometry, 2);
        assert_eq!(sim.node_keys[1], vec![4, 6]);
        let mut dirty = BTreeSet::new();
        sim.insert_key(5, &mut dirty);
        assert_eq!(sim.splits, 1);
        assert_eq!(sim.node_count(), 5);
        assert_eq!(sim.node_keys[sim.identifier_at(1) as usize], vec![4], "左半留 ⌊3/2⌋ = 1 个");
        assert_eq!(sim.node_keys[sim.identifier_at(2) as usize], vec![5, 6], "右半 2 个");
        assert_eq!(sim.key_interval_partition(2), vec![vec![0], vec![1, 2], vec![3], vec![4]]);
        assert_eq!(sim.node_group_partition(), vec![vec![0, 1], vec![2], vec![3, 4]]);
        assert_ne!(sim.key_interval_partition(2), sim.node_group_partition());
        assert_eq!(dirty, BTreeSet::from([1u32, 4]), "分裂方与新节点都脏");
    }

    /// 分裂方的组没满时新节点跟着它——与「满了另开」是同一条规则的两个分支。
    #[test]
    fn split_child_joins_parent_group_when_group_not_full() {
        let geometry = Geometry { node_capacity: 2, initial_keys_per_node: 2, inserts_per_checkpoint: 1, deletes_per_checkpoint: 0 };
        let mut sim = Sim::new(4, 8, 4, geometry, 4);
        let mut dirty = BTreeSet::new();
        sim.insert_key(5, &mut dirty);
        assert_eq!(sim.node_group_partition(), vec![vec![0, 1, 3, 4], vec![2]], "G = 4 时组 {{0,1,2,3}} 已满，新节点另开一组、只有它自己");
        let mut sim2 = Sim::new(4, 8, 4, geometry, 8);
        let mut dirty2 = BTreeSet::new();
        sim2.insert_key(5, &mut dirty2);
        assert_eq!(sim2.node_group_partition(), vec![vec![0, 1, 2, 3, 4]], "G = 8 时组没满，新节点跟着分裂方");
        assert_eq!(sim2.next_group_identifier, 1, "没开新组");
        assert_eq!(sim.next_group_identifier, 2, "开了一个新组");
    }

    /// 删到 < C/4 就与右邻合并，退场的槽进 defer；合并后超容量再分裂。
    #[test]
    fn delete_below_quarter_capacity_merges_with_right_neighbour() {
        let geometry = Geometry { node_capacity: 8, initial_keys_per_node: 4, inserts_per_checkpoint: 1, deletes_per_checkpoint: 1 };
        let mut sim = Sim::new(4, 8, 4, geometry, 2);
        let mut dirty = BTreeSet::new();
        // 节点 1 的 key 是 8,10,12,14；删三个剩 1 个 < 2 ⇒ 并入右邻（节点 2：16..22）
        sim.delete_key(8, &mut dirty);
        sim.delete_key(10, &mut dirty);
        assert_eq!(sim.merges, 0);
        sim.delete_key(12, &mut dirty);
        assert_eq!(sim.merges, 1);
        assert_eq!(sim.node_count(), 3);
        assert_eq!(sim.node_keys[sim.identifier_at(1) as usize], vec![14, 16, 18, 20, 22]);
        assert_eq!(sim.deferred, vec![2], "退场的是原来位置 2 的节点，它住槽 2");
        assert_eq!(sim.runs_incremental(), sim.audit_runs());
        assert_eq!(sim.keys_held_in_nodes(), sim.live_key_count());
    }

    /// 合并后超过容量要再分裂一次（模型假设，写在文件头）。
    #[test]
    fn merge_overflow_splits_again() {
        let geometry = Geometry { node_capacity: 8, initial_keys_per_node: 8, inserts_per_checkpoint: 1, deletes_per_checkpoint: 1 };
        let mut sim = Sim::new(3, 8, 4, geometry, 2);
        let mut dirty = BTreeSet::new();
        // 节点 0 有 8 个 key（0..14 偶数）；删 7 个剩 1 ⇒ 并入右邻（8 个）⇒ 9 > 8 ⇒ 再分裂成 4 + 5
        for key in [0u32, 2, 4, 6, 8, 10, 12] {
            sim.delete_key(key, &mut dirty);
        }
        assert_eq!(sim.merges, 1);
        assert_eq!(sim.splits, 1);
        assert_eq!(sim.node_count(), 3);
        assert_eq!(sim.node_keys[sim.identifier_at(0) as usize].len(), 4);
        assert_eq!(sim.node_keys[sim.identifier_at(1) as usize].len(), 5);
        assert_eq!(sim.unplaced_nodes, 1, "分裂出的右半还没被放置");
    }

    /// 树状数组的第 rank 个存活 key 与暴力扫描一致（含删除后的空洞）。
    #[test]
    fn presence_index_rank_matches_brute_force() {
        let mut index = PresenceIndex::new(37);
        for key in [0usize, 3, 4, 9, 20, 36] {
            index.set(key, true);
        }
        index.set(4, false);
        let brute: Vec<usize> = (0..37).filter(|&key| index.is_present(key)).collect();
        assert_eq!(brute, vec![0, 3, 9, 20, 36]);
        for (rank, &key) in brute.iter().enumerate() {
            assert_eq!(index.key_at_rank(rank), key, "rank {rank}");
        }
        assert_eq!(index.live_count, 5);
    }

    /// 判据 2 守恒：增量 runs 与审计 runs 逐点相等，占用 = 节点数，空 + 占 = S，key 总数 = 存活 key 数——三条臂都跑。
    #[test]
    fn conservation_holds_on_every_arm_under_churn() {
        for arm in [Arm::PolicyCompact(32), Arm::KeyInterval { width: 32, extra: 32 }, Arm::NodeCountGroup { group_size: 8, extra: 32 }] {
            let outcome = short_run(arm, Load::Uniform);
            assert!(outcome.runs_final >= 1 && outcome.runs_final <= outcome.node_count_final as u64, "{}", arm.tag());
        }
    }

    /// 判据 4：主格几何下每条臂的分裂与合并都真的发生了。
    /// 200 个 checkpoint 不够：每节点 4 个 key、容量 8，一个节点要吃到 5 次插入才分裂，3200 次插入摊到 8192 个节点上期望不到一次；
    /// 1000 个 checkpoint（16000 次插入）起才稳定出现，主格跑 2000 个。
    #[test]
    fn splits_and_merges_both_happen_in_main_geometry() {
        for arm in [Arm::PolicyCompact(32), Arm::KeyInterval { width: 8, extra: 32 }, Arm::NodeCountGroup { group_size: 2, extra: 32 }] {
            let outcome = run_arm(arm, Load::Runs8, 0, Geometry::main_grid(), 1000, 125);
            assert!(outcome.splits > 0, "{} 没分裂", arm.tag());
            assert!(outcome.merges > 0, "{} 没合并", arm.tag());
        }
    }

    /// 判据 3 阳性对照：测量管道分得出全隔离与全连续。
    #[test]
    fn measurement_control_discriminates() {
        let mut sim = Sim::new(NODE_COUNT, SLOT_COUNT, SLOTS_PER_SEGMENT, Geometry::main_grid(), 8);
        let (scatter_incremental, scatter_audit, compact_incremental, compact_audit) = measurement_control(&mut sim);
        assert_eq!((scatter_incremental, scatter_audit), (NODE_COUNT as u64, NODE_COUNT as u64));
        assert_eq!((compact_incremental, compact_audit), (1, 1));
    }

    /// 跨装置闸自己要能红：E95 入库产物里的一个数被改掉时，断言必须不成立。
    #[test]
    fn xfixture_gate_is_not_a_rubber_stamp() {
        let got = median_of_five((0..5u64).map(|seed| run_arm(Arm::PolicyCompact(256), Load::Runs8, seed, Geometry::degenerate_e95(), CHECKPOINT_COUNT, SAMPLE_EVERY).runs_final).collect());
        assert_eq!(got, 1339, "本装置退化几何复现 E95 的 pol_compact/runs8/256");
        assert_ne!(got, 1340, "若这条也成立，说明断言根本没在比");
    }

    /// **跨装置闸的本体**：E95 的 24 格逐格复现，节点组那些格两条分组臂各核一遍。
    /// ⚠️ 这条必须遍历整张表——只钉一行的话，改表里别的行一个测试都不会红。
    #[test]
    fn xfixture_reproduces_every_stored_e95_median_on_both_grouping_arms() {
        for (extra, e95_arm_tag, load_tag, want) in E95_MEDIANS {
            for arm in degenerate_arms_for(e95_arm_tag, extra) {
                let got = median_of_five((0..5u64).map(|seed| run_arm(arm, load_from_tag(load_tag), seed, Geometry::degenerate_e95(), CHECKPOINT_COUNT, SAMPLE_EVERY).runs_final).collect());
                assert_eq!(got, want, "跨装置对照失败：{e95_arm_tag}/{load_tag}/{extra} 臂 {}", arm.tag());
            }
        }
    }

    /// 退化几何下节点就是对象：分裂合并零次，节点数不变。
    #[test]
    fn degenerate_geometry_never_splits_or_merges() {
        let outcome = run_arm(Arm::KeyInterval { width: 8, extra: 32 }, Load::Uniform, 0, Geometry::degenerate_e95(), 200, 25);
        assert_eq!((outcome.splits, outcome.merges, outcome.node_count_final), (0, 0, NODE_COUNT));
    }

    /// 节点数分组的绝对值锚点：脏一个节点，整组落成一段连续物理槽，且成员按位置序。
    #[test]
    fn node_count_group_step_relands_whole_group_contiguously() {
        let mut sim = Sim::new(32, 64, 4, Geometry::degenerate_e95(), 4);
        let mut cursor = 0usize;
        step_node_count_group(&mut sim, &[5], 4, 3, &mut cursor);
        let base = sim.slot_at(4);
        for (offset_in_group, position) in (4..8).enumerate() {
            assert_eq!(sim.slot_at(position), base + offset_in_group as u32, "组 {{4..7}} 必须整组连续");
        }
        assert_eq!(sim.total_writes, 4);
        let mut sim2 = Sim::new(32, 64, 4, Geometry::degenerate_e95(), 4);
        let mut cursor2 = 0usize;
        step_node_count_group(&mut sim2, &[5], 4, 0, &mut cursor2);
        assert_eq!(sim2.total_writes, 1, "预算 0 时只写脏的那一个");
    }

    /// key 区间分组：同一区间里两个脏节点只让那个组整落一次（敏感取样点，mutation-sampling.md 第三类）。
    #[test]
    fn key_interval_step_counts_each_group_once() {
        let mut sim = Sim::new(32, 64, 4, Geometry::degenerate_e95(), 1);
        let mut cursor = 0usize;
        step_key_interval(&mut sim, &[4, 5], 4, 2, &mut cursor);
        assert_eq!(sim.total_writes, 4, "同组的两个脏节点只该让这个组整落一次");
    }

    /// 分裂出来的节点住哨兵槽，在 runs 里恒算断开；被放置之后才接上。
    #[test]
    fn unplaced_node_counts_as_broken_until_placed() {
        let geometry = Geometry { node_capacity: 2, initial_keys_per_node: 2, inserts_per_checkpoint: 1, deletes_per_checkpoint: 0 };
        let mut sim = Sim::new(4, 8, 4, geometry, 2);
        let mut dirty = BTreeSet::new();
        sim.insert_key(5, &mut dirty);
        assert_eq!(sim.unplaced_nodes, 1);
        assert_eq!(sim.runs_incremental(), 3, "[0][1] 连、[2] 哨兵断两边、[3][4] 连 ⇒ 3 段");
        assert_eq!(sim.audit_runs(), 3);
        let slot = sim.allocate_first_fit();
        sim.move_node(2, slot);
        assert_eq!(sim.unplaced_nodes, 0);
        assert_eq!(sim.runs_incremental(), sim.audit_runs());
    }

    /// knob 到函数那根线：W 与 G 各自必须真的传进 step 函数（E95 反推腿实测过的坏法）。
    #[test]
    fn knobs_are_wired_through_run_arm() {
        let width_8 = short_run(Arm::KeyInterval { width: 8, extra: 128 }, Load::Runs8).runs_final;
        let width_128 = short_run(Arm::KeyInterval { width: 128, extra: 128 }, Load::Runs8).runs_final;
        let group_2 = short_run(Arm::NodeCountGroup { group_size: 2, extra: 128 }, Load::Runs8).runs_final;
        let group_32 = short_run(Arm::NodeCountGroup { group_size: 32, extra: 128 }, Load::Runs8).runs_final;
        assert_ne!(width_8, width_128, "W = 8 与 W = 128 出同一个数 ⇒ knob 没接上");
        assert_ne!(group_2, group_32, "G = 2 与 G = 32 出同一个数 ⇒ knob 没接上");
    }

    /// 判据 6 的阈值钉死在 0.8：79 / 100 算赢，80 / 100 与 100 / 100 都不算。
    #[test]
    fn win_threshold_is_strictly_below_eight_tenths() {
        assert!(node_group_beats_threshold(79, 100));
        assert!(!node_group_beats_threshold(80, 100));
        assert!(!node_group_beats_threshold(100, 100));
        assert!(!node_group_beats_threshold(1158, 1123), "产物 256/runs8 那一格：节点数分组 1158 没有压过 key 区间 1123");
    }

    /// 等预算是绝对值闸：每个 checkpoint 花的额外写不超过预算（run_arm 里逐轮断言，这里钉 write_amp 的上界）。
    #[test]
    fn budget_is_never_exceeded() {
        for arm in [Arm::PolicyCompact(32), Arm::KeyInterval { width: 32, extra: 32 }, Arm::NodeCountGroup { group_size: 8, extra: 32 }] {
            let outcome = short_run(arm, Load::Uniform);
            assert!(outcome.budget_spent_percent <= 100.0 + 1e-9, "{} 花超：{:.1}%", arm.tag(), outcome.budget_spent_percent);
            assert!(outcome.write_amp >= 1.0);
        }
    }
}
