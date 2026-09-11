//! E96：混合架构的一致性 —— 指针位置条目当提示 + 中央映射为权威，一致性上有没有洞。
//!
//! ## 模型（判据与失败条款的权威登记在 kb/experiments/96-混合架构的一致性.md，跑前写死）
//!
//! 持久 COW 树（叶条目 = (版本, 落点提示)），中央映射 版本→落点 为权威，占用表 落点→版本。
//! 读：提示优先——提示槽的占用者「单元头」与要读的版本匹配就返回，否则回映射多一跳。
//! 单元头两臂：hdr_key（五元组 ⇒ 只有 key）/ hdr_birth（五元组 + 块 birth ⇒ (key, birth)）。
//! 整理只改映射与占用，不碰任何叶的提示。defer：本轮放的槽下一轮才可复用。
//!
//! ## 判据（跑前写死）
//!
//! 1. 手算锚点：快照 → 搬 key 3 → 更新 key 3 复用旧槽 → 读快照 key 3：hdr_key 错读 1，hdr_birth 错读 0 多跳 1。
//! 2. 守恒：被引用的每个版本 v 满足 occupant[map[v]] == v；映射与占用互为反函数。
//! 3. 阳性对照：踩坏映射 ⇒ 审计红；对调占用 ⇒ 错读能响。
//! 4. hdr_key 错读数 == 独立扫描的「提示槽被同 key 另一版本占用」数；hdr_birth 错读恒 0。
//! 5. 按提示释放的误放数 > 0（adversarial）；按映射释放恒 0。
//! 6. 搬迁三步各截断一次：错读恒 0、守恒恒成立。
//! 7. 只报数：活根 / 快照根的多跳率随轮数。

use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet};

const KEY_COUNT: usize = 4096;
const FANOUT: usize = 16;
const SLOT_COUNT: usize = 32768;
const RELOCATION_BATCH_SIZE: usize = 64;
const UPDATES_PER_ROUND: usize = 64;
const ROUNDS: usize = 200;
const SNAPSHOT_EVERY_ROUNDS: usize = 8;
const SNAPSHOTS_RETAINED: usize = 4;

struct XorshiftRandomGenerator(u64);
impl XorshiftRandomGenerator {
    fn new(seed: u64) -> Self {
        let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0xA076_1D64_78BD_642F);
        if state == 0 {
            state = 0xDEAD_BEEF;
        }
        XorshiftRandomGenerator(state)
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, upper_bound_exclusive: usize) -> usize {
        (self.next() % upper_bound_exclusive as u64) as usize
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Header {
    Key,
    KeyBirth,
}
impl Header {
    fn tag(self) -> &'static str {
        match self {
            Header::Key => "hdr_key",
            Header::KeyBirth => "hdr_birth",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AllocationPolicy {
    Lowest, // 最低空槽优先
    Lifo,   // 最近释放的先复用（空闲栈）
}
impl AllocationPolicy {
    fn tag(self) -> &'static str {
        match self {
            AllocationPolicy::Lowest => "alloc_lowest",
            AllocationPolicy::Lifo => "alloc_lifo",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Load {
    Uniform,
    Adversarial,
}
impl Load {
    fn tag(self) -> &'static str {
        match self {
            Load::Uniform => "uniform",
            Load::Adversarial => "adversarial",
        }
    }
}

#[derive(Clone)]
enum Node {
    Inner(Vec<u32>),
    Leaf(Vec<Entry>),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Entry {
    version_number: u32,
    hint: u32,
}

#[derive(Clone, Copy)]
struct Version {
    key: u32,
    birth: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DlEntry {
    version_number: u32,
    hint: u32,
}

#[derive(Clone)]
struct Snapshot {
    txg: u64,
    previous_snapshot_txg: u64,
    root: u32,
    deadlist: Vec<DlEntry>,
}

#[derive(Clone)]
struct World {
    fanout: usize,
    key_count: usize,
    depth: usize,
    header: Header,
    allocation_policy: AllocationPolicy,
    nodes: Vec<Node>,
    live_root: u32,
    snaps: Vec<Snapshot>,
    head_deadlist: Vec<DlEntry>,
    previous_snapshot_txg: u64,
    versions: Vec<Version>,
    free: BTreeSet<u32>,
    /// 释放次序（LIFO 用）：末尾是最近回空的槽。
    free_order: Vec<u32>,
    deferred: Vec<u32>,
    /// 整理的扫描游标：批次取游标起的 N 个占用槽，扫完一圈回头。
    sweep: u32,
    occupant: BTreeMap<u32, u32>,
    map_slot: BTreeMap<u32, u32>,
    txg: u64,
    /// 负臂计数：销毁快照时若按提示解落点会误放几次（只计数，不施加）。
    free_via_hint_misfree: u64,
}

impl World {
    fn new(fanout: usize, key_count: usize, slot_count: usize, header: Header, allocation_policy: AllocationPolicy) -> World {
        let mut depth = 0;
        let mut span = 1;
        while span < key_count {
            span *= fanout;
            depth += 1;
        }
        assert_eq!(span, key_count);
        let mut nodes = Vec::new();
        let mut level: Vec<u32> = (0..key_count / fanout)
            .map(|leaf_ordinal| {
                let leaf_entries: Vec<Entry> =
                    (0..fanout).map(|entry_ordinal| Entry { version_number: (leaf_ordinal * fanout + entry_ordinal) as u32, hint: (leaf_ordinal * fanout + entry_ordinal) as u32 }).collect();
                nodes.push(Node::Leaf(leaf_entries));
                (nodes.len() - 1) as u32
            })
            .collect();
        while level.len() > 1 {
            level = level
                .chunks(fanout)
                .map(|child_chunk| {
                    nodes.push(Node::Inner(child_chunk.to_vec()));
                    (nodes.len() - 1) as u32
                })
                .collect();
        }
        World {
            fanout,
            key_count,
            depth,
            header,
            allocation_policy,
            nodes,
            live_root: level[0],
            snaps: Vec::new(),
            head_deadlist: Vec::new(),
            previous_snapshot_txg: 0,
            versions: (0..key_count).map(|key| Version { key: key as u32, birth: 1 }).collect(),
            free: (key_count as u32..slot_count as u32).collect(),
            free_order: Vec::new(),
            deferred: Vec::new(),
            sweep: 0,
            occupant: (0..key_count as u32).map(|key| (key, key)).collect(),
            map_slot: (0..key_count as u32).map(|key| (key, key)).collect(),
            txg: 1,
            free_via_hint_misfree: 0,
        }
    }

    fn digits(&self, key: usize) -> Vec<usize> {
        let mut span = self.key_count;
        let mut key_digits = Vec::with_capacity(self.depth);
        let mut remaining_key = key;
        for _ in 0..self.depth {
            span /= self.fanout;
            key_digits.push(remaining_key / span);
            remaining_key %= span;
        }
        key_digits
    }

    fn allocate_slot(&mut self) -> u32 {
        let slot = match self.allocation_policy {
            AllocationPolicy::Lowest => *self.free.iter().next().expect("槽用尽"),
            AllocationPolicy::Lifo => loop {
                match self.free_order.pop() {
                    Some(freed_slot) if self.free.contains(&freed_slot) => break freed_slot,
                    Some(_) => continue,
                    None => break *self.free.iter().next().expect("槽用尽"),
                }
            },
        };
        self.free.remove(&slot);
        slot
    }

    /// 释放：进 defer，轮末才回空槽集。
    fn release(&mut self, slot: u32, version_number: u32) {
        assert_eq!(self.occupant.remove(&slot), Some(version_number));
        self.map_slot.remove(&version_number);
        self.deferred.push(slot);
    }

    fn end_round(&mut self) {
        for slot in std::mem::take(&mut self.deferred) {
            self.free.insert(slot);
            self.free_order.push(slot);
        }
    }

    /// 活头写一个 key：新版本、新槽、叶条目提示 = 新槽；kill 旧版本按 D5 规则。
    fn update(&mut self, key: usize) {
        self.txg += 1;
        let slot = self.allocate_slot();
        let version_number = self.versions.len() as u32;
        self.versions.push(Version { key: key as u32, birth: self.txg });
        self.occupant.insert(slot, version_number);
        self.map_slot.insert(version_number, slot);
        let key_digits = self.digits(key);
        let mut path = Vec::with_capacity(self.depth);
        let mut current_node = self.live_root;
        for &digit in &key_digits {
            path.push((current_node, digit));
            current_node = match &self.nodes[current_node as usize] {
                Node::Inner(children) => children[digit],
                Node::Leaf(_) => break,
            };
        }
        let (leaf_node_index, entry_position) = *path.last().unwrap();
        let mut leaf_entries = match &self.nodes[leaf_node_index as usize] {
            Node::Leaf(leaf_entries) => leaf_entries.clone(),
            _ => unreachable!(),
        };
        let old_entry = leaf_entries[entry_position];
        leaf_entries[entry_position] = Entry { version_number, hint: slot };
        self.nodes.push(Node::Leaf(leaf_entries));
        let mut child = (self.nodes.len() - 1) as u32;
        for &(node_index, child_position) in path.iter().rev().skip(1) {
            let mut children = match &self.nodes[node_index as usize] {
                Node::Inner(children) => children.clone(),
                _ => unreachable!(),
            };
            children[child_position] = child;
            self.nodes.push(Node::Inner(children));
            child = (self.nodes.len() - 1) as u32;
        }
        self.live_root = child;
        if self.versions[old_entry.version_number as usize].birth > self.previous_snapshot_txg {
            let old_slot = self.map_slot[&old_entry.version_number];
            self.release(old_slot, old_entry.version_number);
        } else {
            // deadlist 条目带的是叶里那个提示——正是危险所在
            self.head_deadlist.push(DlEntry { version_number: old_entry.version_number, hint: old_entry.hint });
        }
    }

    fn snapshot(&mut self) {
        self.snaps.push(Snapshot {
            txg: self.txg,
            previous_snapshot_txg: self.previous_snapshot_txg,
            root: self.live_root,
            deadlist: std::mem::take(&mut self.head_deadlist),
        });
        self.previous_snapshot_txg = self.txg;
    }

    /// 销毁最旧者（D5 约定 B）。释放**按映射**解落点；同时数一下按提示解会误放几次。
    fn destroy_oldest(&mut self) {
        let oldest_snapshot = self.snaps.remove(0);
        let mut merged = if let Some(next) = self.snaps.first_mut() {
            std::mem::take(&mut next.deadlist)
        } else {
            std::mem::take(&mut self.head_deadlist)
        };
        merged.extend(oldest_snapshot.deadlist);
        let mut kept = Vec::new();
        for deadlist_entry in merged {
            if self.versions[deadlist_entry.version_number as usize].birth > oldest_snapshot.previous_snapshot_txg {
                if self.occupant.get(&deadlist_entry.hint) != Some(&deadlist_entry.version_number) {
                    self.free_via_hint_misfree += 1;
                }
                let slot = self.map_slot[&deadlist_entry.version_number];
                self.release(slot, deadlist_entry.version_number);
            } else {
                kept.push(deadlist_entry);
            }
        }
        if let Some(next) = self.snaps.first_mut() {
            next.deadlist = kept;
            next.previous_snapshot_txg = oldest_snapshot.previous_snapshot_txg;
        } else {
            self.head_deadlist = kept;
        }
    }

    /// 整理：搬一个版本到新槽——只改映射与占用，叶提示不动。
    fn relocate(&mut self, version_number: u32) {
        let old_slot = self.map_slot[&version_number];
        let new_slot = self.allocate_slot();
        self.occupant.insert(new_slot, version_number);
        self.map_slot.insert(version_number, new_slot);
        assert_eq!(self.occupant.remove(&old_slot), Some(version_number));
        self.deferred.push(old_slot);
    }

    /// 单元头匹配：hdr_key 只比 key；hdr_birth 比 (key, birth) ⇒ 等价于同一个版本。
    fn header_matches(&self, occupant_version_number: u32, wanted_version_number: u32) -> bool {
        match self.header {
            Header::Key => self.versions[occupant_version_number as usize].key == self.versions[wanted_version_number as usize].key,
            Header::KeyBirth => occupant_version_number == wanted_version_number,
        }
    }

    fn leaf_entry(&self, root: u32, key: usize) -> Entry {
        let key_digits = self.digits(key);
        let mut current_node = root;
        for &digit in &key_digits[..key_digits.len() - 1] {
            current_node = match &self.nodes[current_node as usize] {
                Node::Inner(children) => children[digit],
                _ => unreachable!(),
            };
        }
        match &self.nodes[current_node as usize] {
            Node::Leaf(leaf_entries) => leaf_entries[*key_digits.last().unwrap()],
            _ => unreachable!(),
        }
    }

    /// 读：提示优先，失配回映射。返回 (读到的版本, 有没有多跳)。
    fn read(&self, root: u32, key: usize) -> (u32, bool) {
        let entry = self.leaf_entry(root, key);
        if let Some(&occupant_version_number) = self.occupant.get(&entry.hint) {
            if self.header_matches(occupant_version_number, entry.version_number) {
                return (occupant_version_number, false);
            }
        }
        let slot = self.map_slot[&entry.version_number];
        (self.occupant[&slot], true)
    }

    /// 读全部根的全部 key：返回 (错读数, 活根多跳数, 快照根多跳数, 快照读总数)。
    fn read_everything(&self) -> (u64, u64, u64, u64) {
        let (mut wrong, mut hop_live, mut hop_snapshot, mut snapshot_reads) = (0u64, 0u64, 0u64, 0u64);
        for key in 0..self.key_count {
            let entry = self.leaf_entry(self.live_root, key);
            let (got, hop) = self.read(self.live_root, key);
            if got != entry.version_number {
                wrong += 1;
            }
            if hop {
                hop_live += 1;
            }
        }
        for snapshot in &self.snaps {
            for key in 0..self.key_count {
                let entry = self.leaf_entry(snapshot.root, key);
                let (got, hop) = self.read(snapshot.root, key);
                snapshot_reads += 1;
                if got != entry.version_number {
                    wrong += 1;
                }
                if hop {
                    hop_snapshot += 1;
                }
            }
        }
        (wrong, hop_live, hop_snapshot, snapshot_reads)
    }

    /// 独立扫描（不走 read）：提示槽被同 key 的**另一个**版本占着的条目数——
    /// hdr_key 臂错读的预测值。
    fn stale_same_key_entries(&self) -> u64 {
        let mut stale_entry_count = 0u64;
        let roots: Vec<u32> = std::iter::once(self.live_root).chain(self.snaps.iter().map(|snapshot| snapshot.root)).collect();
        for root in roots {
            for key in 0..self.key_count {
                let entry = self.leaf_entry(root, key);
                if let Some(&occupant_version_number) = self.occupant.get(&entry.hint) {
                    if occupant_version_number != entry.version_number && self.versions[occupant_version_number as usize].key == self.versions[entry.version_number as usize].key {
                        stale_entry_count += 1;
                    }
                }
            }
        }
        stale_entry_count
    }

    /// 守恒审计：被引用的每个版本 occupant[map[v]] == v；映射与占用互为反函数。
    fn audit(&self) -> Result<(), String> {
        let mut referenced: BTreeSet<u32> = BTreeSet::new();
        let roots: Vec<u32> = std::iter::once(self.live_root).chain(self.snaps.iter().map(|snapshot| snapshot.root)).collect();
        for root in roots {
            for key in 0..self.key_count {
                referenced.insert(self.leaf_entry(root, key).version_number);
            }
        }
        for snapshot in &self.snaps {
            referenced.extend(snapshot.deadlist.iter().map(|deadlist_entry| deadlist_entry.version_number));
        }
        referenced.extend(self.head_deadlist.iter().map(|deadlist_entry| deadlist_entry.version_number));
        for &version_number in &referenced {
            let slot = *self.map_slot.get(&version_number).ok_or(format!("ver {version_number} 无映射"))?;
            if self.occupant.get(&slot) != Some(&version_number) {
                return Err(format!("ver {version_number} 映射到 {slot} 但占用者不是它"));
            }
        }
        for (&slot, &version_number) in &self.occupant {
            if self.map_slot.get(&version_number) != Some(&slot) {
                return Err(format!("槽 {slot} 被 {version_number} 占着但映射不指回来"));
            }
        }
        if self.occupant.len() != self.map_slot.len() {
            return Err("占用数与映射数不等".into());
        }
        Ok(())
    }

    /// 整理批次：从扫描游标起取 N 个占用槽的版本，扫到末尾回头；游标推进到批次末尾之后。
    fn pick_batch(&mut self, batch_size: usize) -> Vec<u32> {
        let mut picked: Vec<(u32, u32)> = self.occupant.range(self.sweep..).take(batch_size).map(|(&slot, &version_number)| (slot, version_number)).collect();
        if picked.len() < batch_size {
            picked.extend(self.occupant.range(..self.sweep).take(batch_size - picked.len()).map(|(&slot, &version_number)| (slot, version_number)));
        }
        self.sweep = picked.last().map(|&(slot, _)| slot + 1).unwrap_or(0);
        picked.into_iter().map(|(_, version_number)| version_number).collect()
    }
}

struct RoundStatistics {
    wrong: u64,
    hop_live_percent: f64,
    hop_snapshot_percent: f64,
}

fn run(header: Header, allocation_policy: AllocationPolicy, load: Load, seed: u64, rounds: usize) -> (World, Vec<RoundStatistics>, u64) {
    let mut world = World::new(FANOUT, KEY_COUNT, SLOT_COUNT, header, allocation_policy);
    let mut random_generator = XorshiftRandomGenerator::new(seed);
    let mut round_statistics = Vec::new();
    let mut last_moved: Vec<u32> = Vec::new();
    let mut wrong_total = 0u64;
    for round_number in 1..=rounds {
        for update_index in 0..UPDATES_PER_ROUND {
            let key = match load {
                Load::Uniform => random_generator.below(KEY_COUNT),
                // 对抗负载：一半更新打刚被搬走的 key
                Load::Adversarial if update_index % 2 == 0 && !last_moved.is_empty() => {
                    world.versions[last_moved[update_index / 2 % last_moved.len()] as usize].key as usize
                }
                Load::Adversarial => random_generator.below(KEY_COUNT),
            };
            world.update(key);
        }
        let batch = world.pick_batch(RELOCATION_BATCH_SIZE);
        for &version_number in &batch {
            world.relocate(version_number);
        }
        last_moved = batch;
        if round_number % SNAPSHOT_EVERY_ROUNDS == 0 {
            world.snapshot();
            if world.snaps.len() > SNAPSHOTS_RETAINED {
                world.destroy_oldest();
            }
        }
        world.end_round();
        world.audit().unwrap_or_else(|error| panic!("轮 {round_number} 守恒破了：{error}"));
        let (wrong, hop_live, hop_snapshot, snapshot_reads) = world.read_everything();
        // 判据 4：错读数与独立扫描逐轮相等（hdr_key）；hdr_birth 恒 0
        match header {
            Header::Key => assert_eq!(wrong, world.stale_same_key_entries(), "轮 {round_number} 错读数与独立扫描分叉"),
            Header::KeyBirth => assert_eq!(wrong, 0, "轮 {round_number} hdr_birth 出现错读"),
        }
        wrong_total += wrong;
        round_statistics.push(RoundStatistics {
            wrong,
            hop_live_percent: 100.0 * hop_live as f64 / KEY_COUNT as f64,
            hop_snapshot_percent: if snapshot_reads > 0 { 100.0 * hop_snapshot as f64 / snapshot_reads as f64 } else { 0.0 },
        });
    }
    (world, round_statistics, wrong_total)
}

/// 判据 6：一次搬迁在三步之后各截断一次，恢复后**不新增**错读（相对截断前的基线）、守恒恒成立。
/// 基线差值而不是绝对 0：hdr_key 臂的世界里本来就可能有提示落到同 key 复用槽的错读，那不是截断造成的。
/// 步骤：① 写新副本（分配 + 占用）② 发布映射 ③ defer 放旧槽。
/// 截断在 ① 之后：未发布的分配回滚（新槽回空、占用去掉）；截断在 ② 之后：旧槽由 defer 放。
fn crash_truncation(base: &World, version_number: u32) -> Vec<(usize, i64)> {
    let (baseline, _, _, _) = base.read_everything();
    let mut wrong_deltas = Vec::new();
    for cut in 1..=3usize {
        let mut world = base.clone();
        let old_slot = world.map_slot[&version_number];
        let new_slot = world.allocate_slot();
        world.occupant.insert(new_slot, version_number); // ① 新副本落盘（占用表暂时两处都有它）
        if cut >= 2 {
            world.map_slot.insert(version_number, new_slot); // ② 发布映射
        }
        if cut >= 3 {
            world.occupant.remove(&old_slot); // ③ 旧槽进 defer
            world.deferred.push(old_slot);
        }
        // 恢复
        if cut == 1 {
            // 未发布的分配回滚
            world.occupant.remove(&new_slot);
            world.free.insert(new_slot);
        } else if cut == 2 {
            // 映射已发布：旧副本按 defer 放
            world.occupant.remove(&old_slot);
            world.deferred.push(old_slot);
        }
        world.end_round();
        world.audit().unwrap_or_else(|error| panic!("截断 {cut} 后守恒破了：{error}"));
        let (wrong, _, _, _) = world.read_everything();
        wrong_deltas.push((cut, wrong as i64 - baseline as i64));
    }
    wrong_deltas
}

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config l={KEY_COUNT} f={FANOUT} s={SLOT_COUNT} n_batch={RELOCATION_BATCH_SIZE} u_per_round={UPDATES_PER_ROUND} rounds={ROUNDS} snap_every={SNAPSHOT_EVERY_ROUNDS} k_snap={SNAPSHOTS_RETAINED} model=counting file_ops=0 seeds=5"
        ))
    );
    for header in [Header::Key, Header::KeyBirth] {
        for allocation_policy in [AllocationPolicy::Lowest, AllocationPolicy::Lifo] {
        for load in [Load::Uniform, Load::Adversarial] {
            for seed in 0..5u64 {
                let (mut world, round_statistics, wrong_total) = run(header, allocation_policy, load, seed, ROUNDS);
                let last_round = round_statistics.last().unwrap();
                let middle_round = &round_statistics[ROUNDS / 2 - 1];
                println!(
                    "{}",
                    emitter.emit_raw(&format!(
                        "name=hybrid hdr={} alloc={} load={} seed={seed} wrong_total={wrong_total} wrong_last={} hop_live_mid={:.1} hop_live_last={:.1} hop_snap_mid={:.1} hop_snap_last={:.1} misfree_via_hint={} misfree_via_map=0",
                        header.tag(),
                        allocation_policy.tag(),
                        load.tag(),
                        last_round.wrong,
                        middle_round.hop_live_percent,
                        last_round.hop_live_percent,
                        middle_round.hop_snapshot_percent,
                        last_round.hop_snapshot_percent,
                        world.free_via_hint_misfree
                    ))
                );
                if seed == 0 {
                    // 判据 6：拿轮末世界里一个被快照引用的版本做三步截断
                    let victim = world.pick_batch(1)[0];
                    let cuts = crash_truncation(&world, victim);
                    let cut_summaries: Vec<String> = cuts.iter().map(|(cut, wrong_delta)| format!("{cut}:{wrong_delta}")).collect();
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "name=crash hdr={} alloc={} load={} seed=0 wrong_delta_after_cut={}",
                            header.tag(),
                            allocation_policy.tag(),
                            load.tag(),
                            cut_summaries.join(",")
                        ))
                    );
                    assert!(cuts.iter().all(|(_, wrong_delta)| *wrong_delta == 0), "截断新增了错读");
                }
            }
        }
        }
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small(header: Header) -> World {
        World::new(2, 8, 32, header, AllocationPolicy::Lowest)
    }

    /// 造判据 1 的场景：快照 → 搬 key 3（槽 3 → 8）→ 轮末放旧槽 → 更新 key 3（新版本复用槽 3）。
    fn hand_scenario(header: Header) -> World {
        let mut world = small(header);
        world.snapshot(); // S1 @ txg 1
        world.relocate(3);
        assert_eq!(world.map_slot[&3], 8);
        world.end_round(); // 槽 3 回空
        world.update(3); // 新版本取最低空槽 = 3
        let live = world.leaf_entry(world.live_root, 3);
        assert_eq!(live.hint, 3, "新版本必须复用了槽 3，场景才成立");
        world
    }

    /// **判据 1（hdr_key）**：快照读 key 3 走提示槽 3，占用者是同 key 的新版本，五元组匹配 ⇒ 静默错读。
    #[test]
    fn hand_key_header_silently_reads_newer_version() {
        let world = hand_scenario(Header::Key);
        let first_snapshot_root = world.snaps[0].root;
        let want = world.leaf_entry(first_snapshot_root, 3).version_number;
        let (got, hop) = world.read(first_snapshot_root, 3);
        assert_eq!(want, 3);
        assert_ne!(got, want, "读到了另一个版本");
        assert!(!hop, "提示「匹配」了所以没多跳——错就错在这里");
        let (wrong, _, _, _) = world.read_everything();
        assert_eq!(wrong, 1);
        assert_eq!(world.stale_same_key_entries(), 1);
    }

    /// **判据 1（hdr_birth）**：同场景，头带 birth ⇒ 提示失配 ⇒ 回映射 ⇒ 读对，多跳 1。
    #[test]
    fn hand_birth_header_falls_back_to_map() {
        let world = hand_scenario(Header::KeyBirth);
        let first_snapshot_root = world.snaps[0].root;
        let (got, hop) = world.read(first_snapshot_root, 3);
        assert_eq!(got, 3);
        assert!(hop);
        let (wrong, hop_live, hop_snapshot, _) = world.read_everything();
        assert_eq!((wrong, hop_live, hop_snapshot), (0, 0, 1));
    }

    /// **判据 5 手算**：deadlist 条目带的是叶提示（槽 3）；S1 销毁时按提示解会放掉新版本正占的槽 3，
    /// 按映射解放的是槽 8。误放计数恰 1，且真正放掉的是 8。
    #[test]
    fn free_via_hint_misfrees_reused_slot() {
        let mut world = hand_scenario(Header::KeyBirth);
        // hand_scenario 里 update(3) 把旧 ver 3（birth 1 ≤ prev_snap 1）推进了活头 deadlist，提示 = 3
        assert_eq!(world.head_deadlist, vec![DlEntry { version_number: 3, hint: 3 }]);
        world.snapshot(); // S2 接管
        world.destroy_oldest(); // 毁 S1：ver 3 释放
        assert_eq!(world.free_via_hint_misfree, 1);
        assert!(world.deferred.contains(&8), "按映射解落点放的是 8");
        assert!(world.occupant.contains_key(&3), "槽 3 仍归新版本");
        world.end_round();
        world.audit().unwrap();
    }

    /// **判据 3**：踩坏一条映射 ⇒ 审计必须红；对调两个槽的占用者 ⇒ 错读计数必须响。
    #[test]
    fn audit_and_wrong_counter_have_teeth() {
        let mut world = small(Header::KeyBirth);
        world.map_slot.insert(3, 9);
        assert!(world.audit().is_err());
        let mut world = small(Header::KeyBirth);
        let occupant_of_slot_3 = world.occupant.remove(&3).unwrap();
        let occupant_of_slot_4 = world.occupant.remove(&4).unwrap();
        world.occupant.insert(3, occupant_of_slot_4);
        world.occupant.insert(4, occupant_of_slot_3);
        let (wrong, _, _, _) = world.read_everything();
        assert_eq!(wrong, 2);
        // 反向核对也要有牙：一个没人引用的版本占着槽 20 而映射说它在 21——
        // 正向只查被引用的版本、长度又相等，只有反向核对能红
        let mut world = small(Header::KeyBirth);
        world.occupant.insert(20, 99);
        world.map_slot.insert(99, 21);
        assert_eq!(world.occupant.len(), world.map_slot.len());
        assert!(world.audit().is_err());
    }

    /// 更新必须刷新自己那条叶条目的提示：活根读刚写的 key 不许多跳，提示 == 映射。
    #[test]
    fn update_refreshes_own_hint() {
        let mut world = small(Header::KeyBirth);
        world.update(3);
        let entry = world.leaf_entry(world.live_root, 3);
        assert_eq!(entry.hint, world.map_slot[&entry.version_number]);
        assert_eq!(entry.hint, 8);
        let (got, hop) = world.read(world.live_root, 3);
        assert_eq!(got, entry.version_number);
        assert!(!hop, "刚写的 key 走提示就该命中");
    }

    /// **判据 6 手算**：8 对象、快照后搬 key 3，三步各截断一次：不新增错读、守恒恒成立
    /// （hdr_key 与 hdr_birth 都跑；hdr_key 的基线本来是 0，所以差值 0 也就是绝对 0）。
    #[test]
    fn crash_truncation_is_clean() {
        for header in [Header::Key, Header::KeyBirth] {
            let mut world = small(header);
            world.snapshot();
            let cuts = crash_truncation(&world, 3);
            assert_eq!(cuts, vec![(1, 0), (2, 0), (3, 0)]);
            let (wrong, _, _, _) = world.read_everything();
            assert_eq!(wrong, 0);
        }
    }

    /// **判据 4 全流程**：LIFO 分配 + adversarial 负载下 hdr_key 的错读总数必须 > 0（危险可达），
    /// hdr_birth 恒 0（run 内逐轮断言），两臂快照读的多跳率都 > 0。
    #[test]
    fn adversarial_load_reaches_the_hazard() {
        let (_, round_statistics, wrong_key) = run(Header::Key, AllocationPolicy::Lifo, Load::Adversarial, 0, 24);
        assert!(wrong_key > 0, "对抗负载没造出同 key 复用");
        assert!(round_statistics.last().unwrap().hop_snapshot_percent > 0.0);
        let (_, round_statistics_birth_header, wrong_birth) = run(Header::KeyBirth, AllocationPolicy::Lifo, Load::Adversarial, 0, 24);
        assert_eq!(wrong_birth, 0);
        assert!(round_statistics_birth_header.last().unwrap().hop_snapshot_percent > 0.0);
    }

    /// 扫描游标：连续两批不重叠且推进，扫完一圈回头。
    #[test]
    fn sweep_cursor_advances_and_wraps() {
        let mut world = small(Header::KeyBirth);
        let first_batch = world.pick_batch(3);
        let second_batch = world.pick_batch(3);
        let third_batch = world.pick_batch(3);
        assert_eq!(first_batch, vec![0, 1, 2]);
        assert_eq!(second_batch, vec![3, 4, 5]);
        assert_eq!(third_batch, vec![6, 7, 0], "扫到末尾回头");
    }

    /// LIFO 分配先复用最近回空的槽：同一轮先搬 2（去 8）再搬 5（去 9），轮末回空次序是 2、5
    /// ⇒ 下一次分配拿 5（最低空槽政策会拿 2），再下一次拿 2，栈空了才退回最低空槽 10。
    #[test]
    fn lifo_reuses_most_recently_freed() {
        let mut world = World::new(2, 8, 32, Header::KeyBirth, AllocationPolicy::Lifo);
        world.relocate(2);
        world.relocate(5);
        assert_eq!((world.map_slot[&2], world.map_slot[&5]), (8, 9));
        world.end_round();
        assert_eq!(world.allocate_slot(), 5, "最近回空的先复用");
        assert_eq!(world.allocate_slot(), 2);
        assert_eq!(world.allocate_slot(), 10, "栈空后退回最低空槽");
    }

    /// 整理只改映射不碰提示：搬完后叶条目的提示还是旧槽。
    #[test]
    fn relocate_leaves_hints_alone() {
        let mut world = small(Header::KeyBirth);
        world.relocate(5);
        assert_eq!(world.leaf_entry(world.live_root, 5).hint, 5);
        assert_eq!(world.map_slot[&5], 8);
        world.end_round();
        world.audit().unwrap();
    }
}
