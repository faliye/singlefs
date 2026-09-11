//! E94：搬共享落点的触碰面 —— D26（后台整理与放置回收）未定项 5 的数据前置。
//!
//! ## 模型（判据与失败条款的权威登记在 kb/experiments/94-搬共享落点的触碰面.md，跑前写死）
//!
//! 一棵扇出 F 的持久 COW 树装 L 个 extent 指针（节点不可变、更新走路径复制、结构共享）。
//! 每 u 次更新立一个快照（保留旧根 + 接管活头 deadlist，D5 的交接形态），保留 K 个。
//! 历史造好后选一批 N 个占用槽当整理批次，三种位置权威架构各搬一遍，数各自改写了什么。
//!
//! | 臂 | 位置权威 | 搬动要改 |
//! |---|---|---|
//! | ptr_rewrite | 每棵引用树指针里的位置条目 | 全部引用叶 COW + 各根涟漪 + 快照根记录 + deadlist 条目 |
//! | central_map | 单点「逻辑身份→物理」映射 | 映射树 COW；主树/deadlist/快照根不动 |
//! | skip_shared | 同 ptr，但只搬引用树恰一棵的 | 可搬集合缩水 |
//! | move_new_birth（负臂） | — | 证明 birth 必须跟逻辑版本走：重指派必出可数违例 |
//!
//! ## 判据（跑前写死）
//!
//! 1. 手算锚点：L=8、F=2 的两个小场景，COW 节点数与根引用更新数钉死（单测）。
//! 2. 语义守恒：搬完后每个保留根逐 key 读回的逻辑内容不变；旧槽零引用；占用数恒等。
//! 3. 阳性对照：注入「漏改一处引用」，审计必须抓到（三臂都过闸）。
//! 4. move_new_birth 的违例数 > 0 且恰等于「被 ≥1 个快照引用且被搬动」的版本数（独立算）。
//! 5. 线性历史下区间判据与树审计必须逐版本一致（D5 的等价性当模型自检用）。
//!
//! ## 它答不了的
//!
//! 计数模型；映射树按「key 空间同几何」近似（叶装该 key 全部代的条目）；
//! deadlist 载体的 COW 不建模（只数条目重写）；无崩溃语义；无读路径代价。

use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet};

const KEY_COUNT: usize = 4096; // key 数（= F^3）
const FANOUT: usize = 16; // 扇出
const SLOT_COUNT: usize = 32768; // 槽数
const COMPACTION_BATCH_SIZE: usize = 64; // 整理批次大小

struct PseudoRandomGenerator(u64);
impl PseudoRandomGenerator {
    fn new(seed: u64) -> Self {
        let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0xA076_1D64_78BD_642F);
        if state == 0 {
            state = 0xDEAD_BEEF;
        }
        PseudoRandomGenerator(state)
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

#[derive(Clone)]
enum Node {
    Inner(Vec<u32>),
    Leaf(Vec<Entry>),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Entry {
    version_index: u32,
    slot: u32,
}

#[derive(Clone, Copy)]
struct Version {
    key: u32,
    birth: u64,
    death: u64, // u64::MAX = 还活着
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DlEntry {
    version_index: u32,
    slot: u32,
}

#[derive(Clone)]
struct Snapshot {
    txg: u64,
    /// 上一个快照的 txg（销毁级联要按 birth > prev(S).txg 过滤，D5 约定 B）。
    previous_snapshot_txg: u64,
    root: u32,
    deadlist: Vec<DlEntry>,
}

#[derive(Clone)]
struct World {
    fanout: usize,
    key_count: usize,
    depth: usize, // 路径节点数（根到叶）
    nodes: Vec<Node>,
    live_root: u32,
    snapshots: Vec<Snapshot>,
    head_deadlist: Vec<DlEntry>,
    previous_snapshot_txg: u64,
    versions: Vec<Version>,
    free_slots: BTreeSet<u32>,
    occupant: BTreeMap<u32, u32>, // slot → ver（分配记录的模型）
    /// central_map 臂的权威映射：ver → slot。ptr 臂不读它。
    map_slot: BTreeMap<u32, u32>,
    txg: u64,
}

impl World {
    fn new(fanout: usize, key_count: usize, slot_count: usize) -> World {
        let mut depth = 0;
        let mut span = 1;
        while span < key_count {
            span *= fanout;
            depth += 1;
        }
        assert_eq!(span, key_count, "L 必须是 F 的整幂");
        let mut nodes = Vec::new();
        // 自底向上建初始树：key k = ver k = 槽 k，birth 1
        let mut level: Vec<u32> = (0..key_count / fanout)
            .map(|leaf_index| {
                let entries: Vec<Entry> = (0..fanout)
                    .map(|position_in_leaf| Entry { version_index: (leaf_index * fanout + position_in_leaf) as u32, slot: (leaf_index * fanout + position_in_leaf) as u32 })
                    .collect();
                nodes.push(Node::Leaf(entries));
                (nodes.len() - 1) as u32
            })
            .collect();
        while level.len() > 1 {
            level = level
                .chunks(fanout)
                .map(|child_group| {
                    nodes.push(Node::Inner(child_group.to_vec()));
                    (nodes.len() - 1) as u32
                })
                .collect();
        }
        let versions: Vec<Version> =
            (0..key_count).map(|key| Version { key: key as u32, birth: 1, death: u64::MAX }).collect();
        let occupant: BTreeMap<u32, u32> = (0..key_count as u32).map(|initial_index| (initial_index, initial_index)).collect();
        let map_slot: BTreeMap<u32, u32> = (0..key_count as u32).map(|initial_index| (initial_index, initial_index)).collect();
        World {
            fanout,
            key_count,
            depth,
            nodes,
            live_root: level[0],
            snapshots: Vec::new(),
            head_deadlist: Vec::new(),
            previous_snapshot_txg: 0,
            versions,
            free_slots: (key_count as u32..slot_count as u32).collect(),
            occupant,
            map_slot,
            txg: 1,
        }
    }

    /// 根到叶的 child 下标序列。
    fn digits(&self, key: usize) -> Vec<usize> {
        let mut span = self.key_count;
        let mut child_indices = Vec::with_capacity(self.depth);
        let mut remaining_key = key;
        for _ in 0..self.depth {
            span /= self.fanout;
            child_indices.push(remaining_key / span);
            remaining_key %= span;
        }
        child_indices
    }

    /// 活头写一个 key：COW 路径复制 + D5 的 kill 规则。
    fn update(&mut self, key: usize) {
        self.txg += 1;
        let slot = *self.free_slots.iter().next().expect("槽用尽");
        self.free_slots.remove(&slot);
        let version_index = self.versions.len() as u32;
        self.versions.push(Version { key: key as u32, birth: self.txg, death: u64::MAX });
        self.occupant.insert(slot, version_index);
        self.map_slot.insert(version_index, slot);

        let child_indices = self.digits(key);
        // 收集路径节点
        let mut path = Vec::with_capacity(self.depth);
        let mut current_node = self.live_root;
        for &child_index in &child_indices {
            path.push((current_node, child_index));
            current_node = match &self.nodes[current_node as usize] {
                Node::Inner(children) => children[child_index],
                Node::Leaf(_) => break,
            };
        }
        // path 最后一项是叶（digits 的最后一位是叶内下标）
        let (leaf_node, index_in_leaf) = *path.last().unwrap();
        let old_entry = match &self.nodes[leaf_node as usize] {
            Node::Leaf(entries) => entries[index_in_leaf],
            _ => unreachable!(),
        };
        // 新叶
        let mut entries = match &self.nodes[leaf_node as usize] {
            Node::Leaf(entries) => entries.clone(),
            _ => unreachable!(),
        };
        entries[index_in_leaf] = Entry { version_index, slot };
        self.nodes.push(Node::Leaf(entries));
        let mut new_child = (self.nodes.len() - 1) as u32;
        // 自底向上复制内部节点
        for &(path_node, child_index) in path.iter().rev().skip(1) {
            let mut children = match &self.nodes[path_node as usize] {
                Node::Inner(children) => children.clone(),
                _ => unreachable!(),
            };
            children[child_index] = new_child;
            self.nodes.push(Node::Inner(children));
            new_child = (self.nodes.len() - 1) as u32;
        }
        self.live_root = new_child;

        // kill 旧版本（D5：birth > previous_snapshot_txg ⇒ 立即释放，否则进活头 deadlist）
        let old_version_index = old_entry.version_index;
        self.versions[old_version_index as usize].death = self.txg;
        if self.versions[old_version_index as usize].birth > self.previous_snapshot_txg {
            self.occupant.remove(&old_entry.slot);
            self.map_slot.remove(&old_version_index);
            self.free_slots.insert(old_entry.slot);
        } else {
            self.head_deadlist.push(DlEntry { version_index: old_version_index, slot: old_entry.slot });
        }
    }

    /// 建快照：保留根 + 接管活头 deadlist（D5 的交接）。
    fn snapshot(&mut self) {
        self.snapshots.push(Snapshot {
            txg: self.txg,
            previous_snapshot_txg: self.previous_snapshot_txg,
            root: self.live_root,
            deadlist: std::mem::take(&mut self.head_deadlist),
        });
        self.previous_snapshot_txg = self.txg;
    }

    /// 销毁最旧的快照（D5 约定 B，对齐 ZFS dsl_destroy 的形态）：
    /// 把被销毁者的 deadlist 并进下一个更新的那侧，然后对**合并结果**过滤——
    /// birth > prev(S).txg 的条目此刻无人引用（只被 S 和更老的引用过），释放。
    /// 只过滤被转移的那半会把「死在下个窗口、只被 S 引用」的块多钉一个窗口。
    fn destroy_oldest(&mut self) {
        let destroyed_snapshot = self.snapshots.remove(0);
        let mut merged = if let Some(next_snapshot) = self.snapshots.first_mut() {
            std::mem::take(&mut next_snapshot.deadlist)
        } else {
            std::mem::take(&mut self.head_deadlist)
        };
        merged.extend(destroyed_snapshot.deadlist);
        let mut kept = Vec::with_capacity(merged.len());
        for entry in merged {
            if self.versions[entry.version_index as usize].birth > destroyed_snapshot.previous_snapshot_txg {
                assert_eq!(self.occupant.remove(&entry.slot), Some(entry.version_index));
                self.map_slot.remove(&entry.version_index);
                self.free_slots.insert(entry.slot);
            } else {
                kept.push(entry);
            }
        }
        if let Some(next_snapshot) = self.snapshots.first_mut() {
            next_snapshot.deadlist = kept;
            next_snapshot.previous_snapshot_txg = destroyed_snapshot.previous_snapshot_txg;
        } else {
            self.head_deadlist = kept;
        }
    }

    /// 守恒：每个被任何保留结构（活树 / 快照树 / deadlist）引用的版本都还占着槽。
    /// 销毁级联把还被引用的槽放掉会在这里现形。
    fn assert_all_references_occupied(&self) {
        let occupied_versions: BTreeSet<u32> = self.occupant.values().copied().collect();
        let roots: Vec<u32> =
            std::iter::once(self.live_root).chain(self.snapshots.iter().map(|snapshot| snapshot.root)).collect();
        for root in roots {
            self.walk(root, &mut |entry| {
                assert!(occupied_versions.contains(&entry.version_index), "树引用的 ver {} 已无槽", entry.version_index);
            });
        }
        for snapshot in &self.snapshots {
            for entry in &snapshot.deadlist {
                assert!(occupied_versions.contains(&entry.version_index), "deadlist 引用的 ver {} 已无槽", entry.version_index);
            }
        }
        for entry in &self.head_deadlist {
            assert!(occupied_versions.contains(&entry.version_index), "活头 deadlist 引用的 ver {} 已无槽", entry.version_index);
        }
        // 反向：每个占着槽的版本都必须被活树、某棵快照树或某张 deadlist 引用——
        // 销毁级联把条目弄丢（槽泄漏）会在这里现形。
        let mut referenced: BTreeSet<u32> = BTreeSet::new();
        for root in
            std::iter::once(self.live_root).chain(self.snapshots.iter().map(|snapshot| snapshot.root))
        {
            self.walk(root, &mut |entry| {
                referenced.insert(entry.version_index);
            });
        }
        for snapshot in &self.snapshots {
            referenced.extend(snapshot.deadlist.iter().map(|entry| entry.version_index));
        }
        referenced.extend(self.head_deadlist.iter().map(|entry| entry.version_index));
        for &version_index in occupied_versions.iter() {
            assert!(referenced.contains(&version_index), "占着槽的 ver {version_index} 无人引用：槽泄漏");
        }
    }

    /// 读回一个根下的逻辑内容：key → ver。
    fn readback(&self, root: u32) -> Vec<u32> {
        let mut version_by_key = Vec::with_capacity(self.key_count);
        self.walk(root, &mut |entry| version_by_key.push(entry.version_index));
        version_by_key
    }

    fn walk(&self, node: u32, visit_entry: &mut impl FnMut(Entry)) {
        match &self.nodes[node as usize] {
            Node::Inner(children) => {
                for &child in children {
                    self.walk(child, visit_entry);
                }
            }
            Node::Leaf(entries) => {
                for &entry in entries {
                    visit_entry(entry);
                }
            }
        }
    }

    /// 树审计：每个版本被哪些快照的树引用（独立于区间判据的第二条路）。
    fn snapshot_references_by_tree(&self) -> BTreeMap<u32, BTreeSet<usize>> {
        let mut references: BTreeMap<u32, BTreeSet<usize>> = BTreeMap::new();
        for (snapshot_index, snapshot) in self.snapshots.iter().enumerate() {
            self.walk(snapshot.root, &mut |entry| {
                references.entry(entry.version_index).or_default().insert(snapshot_index);
            });
        }
        references
    }

    /// 区间判据（D5 的引用条件）：birth ≤ S.txg < death。
    fn snapshot_references_by_interval(&self, version_index: u32) -> BTreeSet<usize> {
        let version = self.versions[version_index as usize];
        self.snapshots
            .iter()
            .enumerate()
            .filter(|(_, snapshot)| version.birth <= snapshot.txg && snapshot.txg < version.death)
            .map(|(snapshot_index, _)| snapshot_index)
            .collect()
    }

    /// 判据 5：线性历史下区间判据与树审计逐版本一致（模型自检）。
    fn assert_interval_matches_tree(&self) {
        let references_by_tree = self.snapshot_references_by_tree();
        for (&slot, &version_index) in &self.occupant {
            let _ = slot;
            let by_tree = references_by_tree.get(&version_index).cloned().unwrap_or_default();
            let by_interval = self.snapshot_references_by_interval(version_index);
            assert_eq!(by_tree, by_interval, "ver={version_index} 区间判据与树审计分叉");
        }
    }
}

/// 一次搬迁的触碰面。
#[derive(Default, Debug, PartialEq, Eq)]
struct Cost {
    copy_on_write_nodes: u64,     // 新写出的树节点数（ptr：引用树；central：映射树）
    root_references: u64,     // 要重写的根引用数（活根记录 + 快照列表条目）
    deadlist_rewrites: u64,   // deadlist 条目重写数
    moved: u64,         // 实际搬动的版本数
    skipped_shared: u64, // skip_shared 臂放弃的版本数
}

/// 给每个要搬的版本分配新槽（取最低空槽；destinations 不影响计数结论）。
fn plan_moves(world: &World, version_indices: &[u32]) -> BTreeMap<u32, (u32, u32)> {
    // ver → (旧槽, 新槽)
    let current_slot_by_version: BTreeMap<u32, u32> = world.occupant.iter().map(|(&slot, &version_index)| (version_index, slot)).collect();
    let mut free_slots_ascending = world.free_slots.iter().copied();
    version_indices.iter().map(|&version_index| (version_index, (current_slot_by_version[&version_index], free_slots_ascending.next().expect("槽用尽")))).collect()
}

/// ptr_rewrite：对每个根做记忆化重建波，返回新节点数与被改根数；同时改写 deadlist 与分配记录。
fn pointer_rewrite_move(world: &mut World, plan: &BTreeMap<u32, (u32, u32)>) -> Cost {
    let mut memoized_rebuilds: BTreeMap<u32, Option<u32>> = BTreeMap::new();
    let mut created_nodes = 0u64;
    let mut root_references = 0u64;
    let roots: Vec<u32> =
        std::iter::once(world.live_root).chain(world.snapshots.iter().map(|snapshot| snapshot.root)).collect();
    let mut new_roots = Vec::with_capacity(roots.len());
    for root in roots {
        let rebuilt_root = rebuild(world, root, plan, &mut memoized_rebuilds, &mut created_nodes);
        if rebuilt_root.is_some() {
            root_references += 1;
        }
        new_roots.push(rebuilt_root);
    }
    if let Some(new_root) = new_roots[0] {
        world.live_root = new_root;
    }
    for (snapshot_index, new_root) in new_roots[1..].iter().enumerate() {
        if let Some(new_root) = new_root {
            world.snapshots[snapshot_index].root = *new_root;
        }
    }
    let mut deadlist_rewrites = 0u64;
    for snapshot in &mut world.snapshots {
        for entry in &mut snapshot.deadlist {
            if let Some(&(_, new_slot)) = plan.get(&entry.version_index) {
                entry.slot = new_slot;
                deadlist_rewrites += 1;
            }
        }
    }
    for entry in &mut world.head_deadlist {
        if let Some(&(_, new_slot)) = plan.get(&entry.version_index) {
            entry.slot = new_slot;
            deadlist_rewrites += 1;
        }
    }
    apply_allocation_records(world, plan);
    Cost { copy_on_write_nodes: created_nodes, root_references, deadlist_rewrites: deadlist_rewrites, moved: plan.len() as u64, skipped_shared: 0 }
}

/// 记忆化重建：返回 Some(新节点) 当且仅当子树里有被搬的版本。
fn rebuild(
    world: &mut World,
    node: u32,
    plan: &BTreeMap<u32, (u32, u32)>,
    memoized_rebuilds: &mut BTreeMap<u32, Option<u32>>,
    created_nodes: &mut u64,
) -> Option<u32> {
    if let Some(&memoized_result) = memoized_rebuilds.get(&node) {
        return memoized_result;
    }
    let result = match world.nodes[node as usize].clone() {
        Node::Leaf(mut entries) => {
            let mut subtree_changed = false;
            for entry in &mut entries {
                if let Some(&(_, new_slot)) = plan.get(&entry.version_index) {
                    entry.slot = new_slot;
                    subtree_changed = true;
                }
            }
            if subtree_changed {
                world.nodes.push(Node::Leaf(entries));
                *created_nodes += 1;
                Some((world.nodes.len() - 1) as u32)
            } else {
                None
            }
        }
        Node::Inner(children) => {
            let mut new_children = children.clone();
            let mut subtree_changed = false;
            for (child_position, &child) in children.iter().enumerate() {
                if let Some(new_child) = rebuild(world, child, plan, memoized_rebuilds, created_nodes) {
                    new_children[child_position] = new_child;
                    subtree_changed = true;
                }
            }
            if subtree_changed {
                world.nodes.push(Node::Inner(new_children));
                *created_nodes += 1;
                Some((world.nodes.len() - 1) as u32)
            } else {
                None
            }
        }
    };
    memoized_rebuilds.insert(node, result);
    result
}

/// central_map：映射树（key 空间同几何、单根）的记忆化波；主树、deadlist、快照根一律不动。
fn central_map_move(world: &mut World, plan: &BTreeMap<u32, (u32, u32)>) -> Cost {
    // 触到的映射叶 = 被搬版本的 key 按 F 分组；逐层向上并到单根
    let mut level: BTreeSet<usize> =
        plan.keys().map(|&version_index| world.versions[version_index as usize].key as usize / world.fanout).collect();
    let mut created_nodes = level.len() as u64;
    let mut span = world.key_count / world.fanout;
    while span > 1 {
        span /= world.fanout;
        level = level.iter().map(|&mapping_node_index| mapping_node_index / world.fanout).collect();
        created_nodes += level.len() as u64;
    }
    for (&version_index, &(_, new_slot)) in plan {
        world.map_slot.insert(version_index, new_slot);
    }
    apply_allocation_records(world, plan);
    Cost {
        copy_on_write_nodes: created_nodes,
        root_references: 1,
        deadlist_rewrites: 0,
        moved: plan.len() as u64,
        skipped_shared: 0,
    }
}

/// 分配记录与占用表跟着搬（两臂共用的那笔账，判据 6 单列）。
fn apply_allocation_records(world: &mut World, plan: &BTreeMap<u32, (u32, u32)>) {
    for (&version_index, &(old_slot, new_slot)) in plan {
        assert_eq!(world.occupant.remove(&old_slot), Some(version_index));
        world.free_slots.insert(old_slot);
        let was_free_before = world.free_slots.remove(&new_slot);
        assert!(was_free_before, "新槽必须来自空槽集");
        world.occupant.insert(new_slot, version_index);
    }
}

/// 搬迁后的引用审计：位置权威侧的每一处引用都必须指到新槽，旧槽零引用。
/// 返回「还指着旧槽的引用数」。与搬迁实现不共享改写代码。
fn audit_stale_references(world: &World, architecture: Architecture, plan: &BTreeMap<u32, (u32, u32)>) -> u64 {
    let old_slots: BTreeSet<u32> = plan.values().map(|&(old_slot, _)| old_slot).collect();
    let mut stale_reference_count = 0u64;
    match architecture {
        Architecture::PointerRewrite | Architecture::SkipShared => {
            let roots: Vec<u32> =
                std::iter::once(world.live_root).chain(world.snapshots.iter().map(|snapshot| snapshot.root)).collect();
            // 同一物理节点只审一次（结构共享下的去重——审计口径与盘上字节一致）
            let mut audited_nodes: BTreeSet<u32> = BTreeSet::new();
            for root in roots {
                audit_walk(world, root, &old_slots, &mut audited_nodes, &mut stale_reference_count);
            }
            for snapshot in &world.snapshots {
                for entry in &snapshot.deadlist {
                    if old_slots.contains(&entry.slot) {
                        stale_reference_count += 1;
                    }
                }
            }
            for entry in &world.head_deadlist {
                if old_slots.contains(&entry.slot) {
                    stale_reference_count += 1;
                }
            }
        }
        Architecture::CentralMap => {
            for (&_version_index, &slot) in &world.map_slot {
                if old_slots.contains(&slot) {
                    stale_reference_count += 1;
                }
            }
        }
    }
    stale_reference_count
}

fn audit_walk(world: &World, node: u32, old_slots: &BTreeSet<u32>, audited_nodes: &mut BTreeSet<u32>, stale_reference_count: &mut u64) {
    if !audited_nodes.insert(node) {
        return;
    }
    match &world.nodes[node as usize] {
        Node::Inner(children) => {
            for &child in children {
                audit_walk(world, child, old_slots, audited_nodes, stale_reference_count);
            }
        }
        Node::Leaf(entries) => {
            for entry in entries {
                if old_slots.contains(&entry.slot) {
                    *stale_reference_count += 1;
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Architecture {
    PointerRewrite,
    CentralMap,
    SkipShared,
}
impl Architecture {
    fn output_label(self) -> &'static str {
        match self {
            Architecture::PointerRewrite => "ptr_rewrite",
            Architecture::CentralMap => "central_map",
            Architecture::SkipShared => "skip_shared",
        }
    }
}

/// 造历史：K 个快照、快照间 u 次均匀随机 key 更新；
/// 最后一个快照之后再走 u 次更新——没有这一段，历史恰好停在快照边界上，
/// 所有活版本 birth ≤ 最新快照 txg ⇒ 全部被快照引用，skip_shared 的可搬集合恒空
/// （那是建模伪影，不是它的天花板）。
fn build_history(retained_snapshots: usize, updates_per_snapshot: usize, seed: u64) -> World {
    let mut world = World::new(FANOUT, KEY_COUNT, SLOT_COUNT);
    let mut random_generator = PseudoRandomGenerator::new(seed);
    // k + 4 轮里滚动保留 k 个快照：不销毁的话盘底沦为「被死快照钉住的化石层」，
    // 那是「从不销毁」的伪影，不是保留窗为 k 的稳态。
    for _ in 0..retained_snapshots + 4 {
        for _ in 0..updates_per_snapshot {
            let key = random_generator.below(KEY_COUNT);
            world.update(key);
        }
        world.snapshot();
        if world.snapshots.len() > retained_snapshots {
            world.destroy_oldest();
        }
    }
    for _ in 0..updates_per_snapshot {
        let key = random_generator.below(KEY_COUNT);
        world.update(key);
    }
    world.assert_all_references_occupied();
    world.assert_interval_matches_tree();
    world
}

/// 批次：phys = 槽号最低的 N 个占用槽；keyclu = key 连续段的现行活版本。
fn pick_batch(world: &World, mode: &str) -> Vec<u32> {
    match mode {
        "phys" => world.occupant.values().take(COMPACTION_BATCH_SIZE).copied().collect(),
        "keyclu" => {
            let live_version_by_key = world.readback(world.live_root);
            (1024..1024 + COMPACTION_BATCH_SIZE).map(|key| live_version_by_key[key]).collect()
        }
        _ => unreachable!(),
    }
}

/// 批次里 key 的分散度：把批次版本的 key 排序后数极大连续段。
fn key_runs(world: &World, version_indices: &[u32]) -> u64 {
    let mut keys: Vec<u32> = version_indices.iter().map(|&version_index| world.versions[version_index as usize].key).collect();
    keys.sort_unstable();
    keys.dedup();
    let mut runs = 1u64;
    for sorted_position in 1..keys.len() {
        if keys[sorted_position] != keys[sorted_position - 1] + 1 {
            runs += 1;
        }
    }
    runs
}

/// 跑一个架构臂：clone 世界、搬、审计守恒。返回触碰面。
fn run_architecture(base: &World, architecture: Architecture, batch: &[u32]) -> Cost {
    let mut world = base.clone();
    let before: Vec<Vec<u32>> = std::iter::once(world.live_root)
        .chain(world.snapshots.iter().map(|snapshot| snapshot.root))
        .map(|root| world.readback(root))
        .collect();
    let occupied_before = world.occupant.len();

    let (version_indices, skipped): (Vec<u32>, u64) = match architecture {
        Architecture::SkipShared => {
            let references_by_tree = world.snapshot_references_by_tree();
            let movable: Vec<u32> =
                batch.iter().copied().filter(|version_index| !references_by_tree.contains_key(version_index)).collect();
            let skipped = (batch.len() - movable.len()) as u64;
            (movable, skipped)
        }
        _ => (batch.to_vec(), 0),
    };
    let slots_total = world.occupant.len() + world.free_slots.len();
    let plan = plan_moves(&world, &version_indices);
    let mut cost = match architecture {
        Architecture::PointerRewrite | Architecture::SkipShared => pointer_rewrite_move(&mut world, &plan),
        Architecture::CentralMap => central_map_move(&mut world, &plan),
    };
    cost.skipped_shared = skipped;
    assert_eq!(world.occupant.len() + world.free_slots.len(), slots_total, "{architecture:?} 槽守恒破了");

    // 判据 2：语义守恒 + 旧槽零引用 + 占用数恒等
    let after: Vec<Vec<u32>> = std::iter::once(world.live_root)
        .chain(world.snapshots.iter().map(|snapshot| snapshot.root))
        .map(|root| world.readback(root))
        .collect();
    assert_eq!(before, after, "{architecture:?} 搬迁改了逻辑内容");
    assert_eq!(world.occupant.len(), occupied_before, "{architecture:?} 占用数变了");
    assert_eq!(audit_stale_references(&world, architecture, &plan), 0, "{architecture:?} 旧槽还有引用");
    cost
}

/// 判据 3 阳性对照：漏改一处引用，审计必须抓到。
fn injection_control(base: &World, architecture: Architecture, batch: &[u32]) {
    let mut world = base.clone();
    let version_indices: Vec<u32> = match architecture {
        Architecture::SkipShared => {
            let references_by_tree = world.snapshot_references_by_tree();
            batch.iter().copied().filter(|version_index| !references_by_tree.contains_key(version_index)).collect()
        }
        _ => batch.to_vec(),
    };
    if version_indices.is_empty() {
        return;
    }
    let plan = plan_moves(&world, &version_indices);
    match architecture {
        Architecture::PointerRewrite | Architecture::SkipShared => {
            pointer_rewrite_move(&mut world, &plan);
            // 漏改：把活根路径上第一个含被搬版本的叶改回旧槽
            let (&version_index, &(old_slot, _)) = plan.iter().next().unwrap();
            let key = world.versions[version_index as usize].key as usize;
            let child_indices = world.digits(key);
            let mut current_node = world.live_root;
            for &child_index in &child_indices[..child_indices.len() - 1] {
                current_node = match &world.nodes[current_node as usize] {
                    Node::Inner(children) => children[child_index],
                    _ => unreachable!(),
                };
            }
            if let Node::Leaf(entries) = &mut world.nodes[current_node as usize] {
                entries[*child_indices.last().unwrap()].slot = old_slot;
            }
        }
        Architecture::CentralMap => {
            central_map_move(&mut world, &plan);
            let (&version_index, &(old_slot, _)) = plan.iter().next().unwrap();
            world.map_slot.insert(version_index, old_slot);
        }
    }
    let stale_reference_count = audit_stale_references(&world, architecture, &plan);
    assert!(stale_reference_count > 0, "{architecture:?} 注入的漏改没被审计抓到：整轮作废");
}

/// 判据 4 负臂：搬动时重指派 birth，违例数必须恰等于「被快照树引用且被搬动」的版本数。
fn new_birth_violations(base: &World, batch: &[u32]) -> (u64, u64) {
    let mut world = base.clone();
    let references_by_tree = world.snapshot_references_by_tree();
    let expected: u64 = batch.iter().filter(|version_index| references_by_tree.contains_key(version_index)).count() as u64;
    let plan = plan_moves(&world, &batch.to_vec());
    pointer_rewrite_move(&mut world, &plan);
    let reassigned_birth = world.txg + 1;
    for &version_index in batch {
        world.versions[version_index as usize].birth = reassigned_birth;
    }
    // 违例：树审计说被引用、区间判据说没人引用
    let mut violations = 0u64;
    for &version_index in batch {
        let by_tree = references_by_tree.get(&version_index).cloned().unwrap_or_default();
        let by_interval = world.snapshot_references_by_interval(version_index);
        if !by_tree.is_empty() && by_interval.is_empty() {
            violations += 1;
        }
    }
    (violations, expected)
}

fn main() {
    let mut emitter = Emitter::new();
    println!(
        "{}",
        emitter.emit_raw(&format!(
            "name=config l={KEY_COUNT} f={FANOUT} s={SLOT_COUNT} n_batch={COMPACTION_BATCH_SIZE} model=counting file_ops=0 seeds=5"
        ))
    );

    for retained_snapshots in [1usize, 4, 16] {
        for updates_per_snapshot in [64usize, 1024] {
            for mode in ["phys", "keyclu"] {
                for seed in 0..5u64 {
                    let world = build_history(retained_snapshots, updates_per_snapshot, seed);
                    let batch = pick_batch(&world, mode);
                    let batch_key_runs = key_runs(&world, &batch);
                    for architecture in [Architecture::PointerRewrite, Architecture::CentralMap, Architecture::SkipShared] {
                        injection_control(&world, architecture, &batch);
                        let cost = run_architecture(&world, architecture, &batch);
                        println!(
                            "{}",
                            emitter.emit_raw(&format!(
                                "name=move arch={} k={retained_snapshots} u={updates_per_snapshot} batch={mode} seed={seed} key_runs={batch_key_runs} moved={} cow_nodes={} root_refs={} dl_rewrites={} skipped={}",
                                architecture.output_label(),
                                cost.moved,
                                cost.copy_on_write_nodes,
                                cost.root_references,
                                cost.deadlist_rewrites,
                                cost.skipped_shared
                            ))
                        );
                    }
                    let (violations, expected) = new_birth_violations(&world, &batch);
                    assert_eq!(violations, expected, "违例数必须恰等于独立算出的破坏数");
                    assert!(
                        violations > 0 || expected == 0,
                        "k={retained_snapshots} u={updates_per_snapshot} {mode} seed={seed}：负臂零违例且预期非零"
                    );
                    println!(
                        "{}",
                        emitter.emit_raw(&format!(
                            "name=new_birth k={retained_snapshots} u={updates_per_snapshot} batch={mode} seed={seed} violations={violations}"
                        ))
                    );
                }
            }
        }
    }
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_world() -> World {
        World::new(2, 8, 32)
    }

    /// **判据 1 手算锚点（共享整树）**：L=8、F=2，快照后零更新 ⇒ 快照根 == 活根（同一节点）。
    /// 搬 key 3 的版本：唯一叶副本 + 1 个内部 + 1 个根 = 3 个新节点；
    /// 根引用更新 = 2（活根记录 + 快照列表条目指向同一个旧根，都要改指新根）。
    #[test]
    fn hand_shared_whole_tree() {
        let mut world = small_world();
        world.snapshot();
        assert_eq!(world.snapshots[0].root, world.live_root);
        let plan = plan_moves(&world, &[3]);
        let cost = pointer_rewrite_move(&mut world, &plan);
        assert_eq!(cost.copy_on_write_nodes, 3);
        assert_eq!(cost.root_references, 2);
        assert_eq!(cost.deadlist_rewrites, 0);
        assert_eq!(audit_stale_references(&world, Architecture::PointerRewrite, &plan), 0);
    }

    /// **判据 1 手算锚点（部分分叉）**：快照后更新 key 0 ⇒ 活树左半 COW。
    /// 搬 key 3：叶 {2,3} 仍是单副本，但它的父辈在两棵树里已分叉
    /// ⇒ 1 叶 + 2 个内部（快照侧 + 活侧）+ 2 个根 = 5 个新节点；根引用 2。
    #[test]
    fn hand_diverged_paths() {
        let mut world = small_world();
        world.snapshot();
        world.update(0);
        let plan = plan_moves(&world, &[3]);
        let cost = pointer_rewrite_move(&mut world, &plan);
        assert_eq!(cost.copy_on_write_nodes, 5);
        assert_eq!(cost.root_references, 2);
        assert_eq!(audit_stale_references(&world, Architecture::PointerRewrite, &plan), 0);
    }

    /// **判据 1 手算锚点（central_map）**：同场景搬 key 3：映射树 1 叶 + 1 内部 + 1 根 = 3 节点、
    /// 根引用 1、deadlist 0；主树一个节点都不写。
    #[test]
    fn hand_central() {
        let mut world = small_world();
        world.snapshot();
        world.update(0);
        let nodes_before = world.nodes.len();
        let plan = plan_moves(&world, &[3]);
        let cost = central_map_move(&mut world, &plan);
        assert_eq!(cost.copy_on_write_nodes, 3);
        assert_eq!(cost.root_references, 1);
        assert_eq!(cost.deadlist_rewrites, 0);
        assert_eq!(world.nodes.len(), nodes_before, "主树一个节点都不许写");
        assert_eq!(audit_stale_references(&world, Architecture::CentralMap, &plan), 0);
    }

    /// **deadlist 条目重写**：快照 1 之后更新 key 3（旧版本 birth 1 ≤ prev_snap ⇒ 进活头 deadlist），
    /// 快照 2 接管它。搬那个死版本：它只被快照 1 的树引用 ⇒ 波只走快照 1 一棵 + deadlist 改 1 条。
    #[test]
    fn dead_version_rewrites_deadlist() {
        let mut world = small_world();
        world.snapshot(); // S1 @ txg 1
        world.update(3); // 旧 ver 3（birth 1）死，进活头 deadlist
        world.snapshot(); // S2 接管 deadlist
        assert_eq!(world.snapshots[1].deadlist.len(), 1);
        let dead_version_index = world.snapshots[1].deadlist[0].version_index;
        assert_eq!(dead_version_index, 3);
        let plan = plan_moves(&world, &[dead_version_index]);
        let cost = pointer_rewrite_move(&mut world, &plan);
        // S1 的树引用它：1 叶 + 1 内部 + 1 根；活树与 S2 树不含它
        assert_eq!(cost.copy_on_write_nodes, 3);
        assert_eq!(cost.root_references, 1);
        assert_eq!(cost.deadlist_rewrites, 1);
        assert_eq!(audit_stale_references(&world, Architecture::PointerRewrite, &plan), 0);
    }

    /// **判据 4 手算**：快照后搬 key 3 并重指派 birth ⇒ 恰 1 个违例（key 3 的版本被快照引用）。
    #[test]
    fn new_birth_breaks_exactly_one() {
        let mut world = small_world();
        world.snapshot();
        let (violations, expected) = new_birth_violations(&world, &[3]);
        assert_eq!(expected, 1);
        assert_eq!(violations, 1);
    }

    /// **判据 5**：随机历史下区间判据与树审计逐版本一致（build_history 内部断言），
    /// 且立即释放规则真的在放槽：快照前的更新对象不进 deadlist。
    #[test]
    fn interval_equals_tree_and_immediate_free() {
        let world = build_history(4, 64, 0);
        assert!(world.snapshots.len() == 4);
        let mut snapshotless_world = World::new(2, 8, 32);
        snapshotless_world.update(3); // 无快照 ⇒ birth 2 > prev_snap 0 ⇒ 立即释放旧槽
        assert!(snapshotless_world.free_slots.contains(&3), "旧槽 3 必须立即回空");
        assert!(snapshotless_world.head_deadlist.is_empty());
    }

    /// skip_shared 只搬未被任何快照引用的：快照后更新 key 0，
    /// 批次 {新 ver(key0), 旧共享 ver(key3)} ⇒ 恰搬 1 个、放弃 1 个。
    #[test]
    fn skip_shared_moves_only_exclusive() {
        let mut world = small_world();
        world.snapshot();
        world.update(0);
        let live_version_by_key = world.readback(world.live_root);
        let batch = vec![live_version_by_key[0], live_version_by_key[3]];
        let cost = run_architecture(&world, Architecture::SkipShared, &batch);
        assert_eq!(cost.moved, 1);
        assert_eq!(cost.skipped_shared, 1);
    }

    /// 注入的漏改必须被审计抓到（三臂）。
    #[test]
    fn injection_is_caught() {
        let world = build_history(2, 16, 1);
        let batch = pick_batch(&world, "phys");
        for architecture in [Architecture::PointerRewrite, Architecture::CentralMap, Architecture::SkipShared] {
            injection_control(&world, architecture, &batch);
        }
    }

    /// 读回守恒经 run_architecture 全流程（三臂 × 两种批次）。
    #[test]
    fn conservation_via_run_architecture() {
        let world = build_history(4, 64, 2);
        for mode in ["phys", "keyclu"] {
            let batch = pick_batch(&world, mode);
            for architecture in [Architecture::PointerRewrite, Architecture::CentralMap, Architecture::SkipShared] {
                let cost = run_architecture(&world, architecture, &batch);
                assert!(cost.moved + cost.skipped_shared == batch.len() as u64);
            }
        }
    }

    /// **销毁级联的手算锚点**：S1 之后更新 key 3（死版本进活头 deadlist）、S2 接管。
    /// 销毁 S1（prev = 0）：合并侧 = S2 的 deadlist，唯一条目 birth 1 > 0 ⇒ 释放——
    /// 旧槽 3 回空、ver 3 不再占槽、S2 的 deadlist 清空。
    /// 只过滤被转移那半的写法会把这条钉到 S2 销毁时才放，这里就红。
    #[test]
    fn destroy_frees_merged_next_side() {
        let mut world = small_world();
        world.snapshot(); // S1 @ txg 1
        world.update(3);
        world.snapshot(); // S2 接管 deadlist（1 条：ver 3 @ 槽 3）
        assert_eq!(world.snapshots[1].deadlist, vec![DlEntry { version_index: 3, slot: 3 }]);
        world.destroy_oldest();
        assert!(world.free_slots.contains(&3), "旧槽 3 必须在 S1 销毁时回空");
        assert!(!world.occupant.contains_key(&3));
        assert!(world.snapshots[0].deadlist.is_empty());
        world.assert_all_references_occupied();
    }

    /// deadlist 审计的判别力：搬完后手工把一条 deadlist 槽位改回旧值，审计必须多出 stale。
    #[test]
    fn deadlist_audit_has_teeth() {
        let mut world = small_world();
        world.snapshot();
        world.update(3);
        world.snapshot();
        let dead_version_index = world.snapshots[1].deadlist[0].version_index;
        let plan = plan_moves(&world, &[dead_version_index]);
        let old_slot = plan[&dead_version_index].0;
        pointer_rewrite_move(&mut world, &plan);
        assert_eq!(audit_stale_references(&world, Architecture::PointerRewrite, &plan), 0);
        world.snapshots[1].deadlist[0].slot = old_slot;
        assert!(audit_stale_references(&world, Architecture::PointerRewrite, &plan) > 0);
    }

    /// **等价性留档（变异表 M5 判等价的依据）**：只销毁最旧者的轮换下，
    /// 被销毁者自己的 deadlist 在销毁那一刻恒空——首个快照的从未装过东西
    /// （它窗口里的 kill 全走「birth > prev=0 ⇒ 立即释放」），其余的在前驱销毁时
    /// 已被「合并下一侧再过滤」清空。所以「把被销毁者的条目并进下一侧」在
    /// 只毁最旧者的序列里是不可达代码；动它的变异等价，不算盲区，分开记。
    #[test]
    fn oldest_first_rotation_means_own_deadlist_already_empty() {
        let mut world = World::new(2, 8, 64);
        let mut random_generator = PseudoRandomGenerator::new(9);
        for _ in 0..6 {
            for _ in 0..4 {
                let key = random_generator.below(8);
                world.update(key);
            }
            world.snapshot();
            if world.snapshots.len() > 2 {
                assert!(
                    world.snapshots[0].deadlist.is_empty(),
                    "最旧者的 deadlist 该已被前驱销毁清空"
                );
                world.destroy_oldest();
            }
        }
        world.assert_all_references_occupied();
    }

    /// **销毁级联的 prev 传递**：毁 S1 后 S2 的 prev 必须变 0，否则毁 S2 时
    /// birth ≤ 旧 prev 的条目被永久钉住。
    #[test]
    fn destroy_cascade_propagates_previous_transaction_group() {
        let mut world = small_world();
        world.snapshot(); // S1 @ txg 1
        world.update(3);
        world.snapshot(); // S2 @ txg 2 接管 {ver3}
        world.update(0);
        world.snapshot(); // S3 @ txg 3 接管 {ver0}
        world.destroy_oldest(); // 毁 S1：ver3 释放
        assert!(world.free_slots.contains(&3));
        world.destroy_oldest(); // 毁 S2：prev 已传递为 0 ⇒ ver0 释放
        assert!(world.free_slots.contains(&0), "prev 不传递的话 ver0 会被钉住");
        world.assert_all_references_occupied();
    }

    /// 批次形状：phys 取槽号最低的 64 个占用槽；keyclu 的 key 恰是一个连续段。
    #[test]
    fn batch_shapes() {
        let world = build_history(4, 64, 3);
        let physical_batch = pick_batch(&world, "phys");
        assert_eq!(physical_batch.len(), 64);
        let key_clustered_batch = pick_batch(&world, "keyclu");
        assert_eq!(key_runs(&world, &key_clustered_batch), 1);
    }
}
