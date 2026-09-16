//! C363（现算保留池时树高从哪读没有条款）第二轮攻方腿，判据 V3：把 K3 的几项按条款归属之后，
//! 空发布（D16（发布语义） 已定项 9：没有需求，全部块都该由 c_max / ckpt_cost 罩着）那一格公式还少算几块。只用 std，确定性。
//!
//! 几何取第一版：两块盘、4 GiB 镜像、单元区 211968 槽（`allocator.rs` 单测 `unit_area_of_a_4_gib_image_has_211968_slots_in_3312_segments`），
//! 两盘同号（`fbae43e` 的 `PoolAllocator::record`）。树是真 B+ 树（插入分裂、不合并），叶容量按 crates/ 的条目宽；
//! 内部节点条目格式除 inode 树外没有条款，按「key + 86 字节子指针」假设，另跑两档内部扇出做敏感性。
//! 与第一轮模型（`c355-c363-r1-opus-model/src/bin/q2_fixpoint.rs`）相比加了四样：单元区按 4 GiB、释放的槽过复用窗口后回收
//! （复用一个有记录的槽 = 原地改写那条分配记录，不插入）、段耗尽回落（D3（空间分配） 已定项 8 第 2 条）、按叶 / 内部拆开数分配记录树的脏节点。

use std::collections::{BTreeMap, BTreeSet, VecDeque};

type Key = u128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SplitPolicy {
    Half,
    RightEndAppend,
}

#[derive(Clone, Debug)]
struct TreeNode {
    is_leaf: bool,
    keys: Vec<Key>,
    children: Vec<usize>,
}

#[derive(Clone, Debug)]
struct BPlusTree {
    leaf_capacity: usize,
    internal_capacity: usize,
    split_policy: SplitPolicy,
    nodes: Vec<TreeNode>,
    root: usize,
    levels: usize,
    splits: u64,
}

impl BPlusTree {
    fn new(leaf_capacity: usize, internal_capacity: usize, split_policy: SplitPolicy) -> Self {
        Self { leaf_capacity, internal_capacity, split_policy, nodes: vec![TreeNode { is_leaf: true, keys: Vec::new(), children: Vec::new() }], root: 0, levels: 1, splits: 0 }
    }
    fn path_to_leaf(&self, key: Key) -> Vec<usize> {
        let mut path = vec![self.root];
        let mut current = self.root;
        while !self.nodes[current].is_leaf {
            let node = &self.nodes[current];
            current = node.children[node.keys.partition_point(|separator| *separator <= key)];
            path.push(current);
        }
        path
    }
    fn touch(&self, key: Key) -> Vec<usize> {
        let path = self.path_to_leaf(key);
        let leaf = *path.last().expect("路径非空");
        assert!(self.nodes[leaf].keys.binary_search(&key).is_ok(), "改写的 key 必须在树里");
        path
    }
    fn remove(&mut self, key: Key) -> Vec<usize> {
        let path = self.path_to_leaf(key);
        let leaf = *path.last().expect("路径非空");
        let position = self.nodes[leaf].keys.binary_search(&key).expect("删的 key 必须在树里");
        self.nodes[leaf].keys.remove(position);
        path
    }
    fn insert(&mut self, key: Key) -> Vec<usize> {
        let path = self.path_to_leaf(key);
        let leaf = *path.last().expect("路径非空");
        let position = self.nodes[leaf].keys.binary_search(&key).expect_err("key 不许重复插入");
        self.nodes[leaf].keys.insert(position, key);
        let mut touched = path.clone();
        let mut depth = path.len();
        while depth > 0 {
            depth -= 1;
            let node_index = path[depth];
            let over = if self.nodes[node_index].is_leaf { self.nodes[node_index].keys.len() > self.leaf_capacity } else { self.nodes[node_index].children.len() > self.internal_capacity };
            if !over {
                break;
            }
            let (separator, right_index) = self.split(node_index, key);
            touched.push(right_index);
            if depth == 0 {
                let new_root = self.nodes.len();
                self.nodes.push(TreeNode { is_leaf: false, keys: vec![separator], children: vec![node_index, right_index] });
                self.root = new_root;
                self.levels += 1;
                touched.push(new_root);
            } else {
                let parent = path[depth - 1];
                let child_position = self.nodes[parent].children.iter().position(|child| *child == node_index).expect("父节点里有它");
                self.nodes[parent].keys.insert(child_position, separator);
                self.nodes[parent].children.insert(child_position + 1, right_index);
            }
        }
        touched
    }
    fn split(&mut self, node_index: usize, inserted_key: Key) -> (Key, usize) {
        self.splits += 1;
        let node = self.nodes[node_index].clone();
        let right_index = self.nodes.len();
        if node.is_leaf {
            let count = node.keys.len();
            let middle = match self.split_policy {
                SplitPolicy::RightEndAppend if *node.keys.last().expect("非空") == inserted_key => count - 1,
                SplitPolicy::RightEndAppend | SplitPolicy::Half => count / 2,
            };
            let right_keys = node.keys[middle..].to_vec();
            let separator = right_keys[0];
            self.nodes[node_index].keys.truncate(middle);
            self.nodes.push(TreeNode { is_leaf: true, keys: right_keys, children: Vec::new() });
            (separator, right_index)
        } else {
            let child_count = node.children.len();
            let inserted_into_last_child = node.keys.last().is_none_or(|last| inserted_key >= *last);
            let middle = match self.split_policy {
                SplitPolicy::RightEndAppend if inserted_into_last_child => child_count - 1,
                SplitPolicy::RightEndAppend | SplitPolicy::Half => child_count / 2,
            };
            let separator = node.keys[middle - 1];
            let right_keys = node.keys[middle..].to_vec();
            let right_children = node.children[middle..].to_vec();
            self.nodes[node_index].keys.truncate(middle - 1);
            self.nodes[node_index].children.truncate(middle);
            self.nodes.push(TreeNode { is_leaf: false, keys: right_keys, children: right_children });
            (separator, right_index)
        }
    }
    fn check_invariants(&self) -> usize {
        fn walk(tree: &BPlusTree, index: usize, depth: usize, low: Option<Key>, high: Option<Key>) -> usize {
            let node = &tree.nodes[index];
            assert!(node.keys.windows(2).all(|pair| pair[0] < pair[1]), "key 无序");
            if let Some(first) = node.keys.first() { assert!(low.is_none_or(|bound| *first >= bound), "key 低于区间"); }
            if let Some(last) = node.keys.last() { assert!(high.is_none_or(|bound| *last < bound), "key 高于区间"); }
            if node.is_leaf {
                assert_eq!(depth, tree.levels, "叶不同深");
                assert!(node.keys.len() <= tree.leaf_capacity, "叶超容量");
                return node.keys.len();
            }
            assert!(node.children.len() <= tree.internal_capacity, "内部节点超容量");
            (0..node.children.len())
                .map(|position| {
                    let child_low = if position == 0 { low } else { Some(node.keys[position - 1]) };
                    let child_high = if position == node.keys.len() { high } else { Some(node.keys[position]) };
                    walk(tree, node.children[position], depth + 1, child_low, child_high)
                })
                .sum()
        }
        walk(self, self.root, 1, None, None)
    }
}

// ---- 第一版几何与 crates/ 的宽度 ----
const UNIT_AREA_START_SLOT: u64 = 50176;
const UNIT_AREA_SLOTS: u64 = 211_968;
const SEGMENT_SLOTS: u64 = 64;
const NODE_BYTES: usize = 16384;
const NODE_POINTER_BYTES: usize = 86;
fn index_node_header_bytes(key_width: usize) -> usize { 86 + 2 * key_width + 29 }
fn leaf_capacity(key_width: usize, entry_width: usize) -> usize { (NODE_BYTES - index_node_header_bytes(key_width)) / entry_width }
fn internal_capacity(key_width: usize) -> usize { (NODE_BYTES - index_node_header_bytes(key_width)) / (key_width + NODE_POINTER_BYTES) }
/// 记账树每行每发布重写（D5（快照 / 空间记账机制） 已定项 2），保留 K = 根环槽总数 + 1 = 8 × 3 + 1 = 25 代，15 行 × 25 = 375 条。
const ACCOUNTING_ROWS_KEPT: usize = 15 * 25;

const TREE_EXTENT: u128 = 11;
const TREE_INODE: u128 = 12;
const TREE_ALLOCATION: u128 = 13;
const TREE_ACCOUNTING: u128 = 14;
const CLASS_DATA: u128 = 1;
const CLASS_INDEX_NODE: u128 = 2;
const CLASS_PACKED: u128 = 3;
const DEVICES: u128 = 2;

fn mapping_key(class: u128, tree: u128, txg: u64, tail: u64) -> Key { (class << 120) | (tree << 104) | (u128::from(txg) << 64) | u128::from(tail) }
fn allocation_key(device: u128, slot: u64) -> Key { (device << 64) | u128::from(slot) }
fn extent_key(offset_units: u64) -> Key { (1u128 << 64) | u128::from(offset_units) }

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TreeName { Extent, Allocation, Accounting, Mapping }

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum NodeId { InTree(TreeName, usize), InodeLeafContainer, InodeRoot, TreeTable }

impl NodeId {
    fn mapping_identity(self) -> Option<(u128, u128)> {
        match self {
            NodeId::InTree(TreeName::Extent, _) => Some((CLASS_INDEX_NODE, TREE_EXTENT)),
            NodeId::InTree(TreeName::Allocation, _) => Some((CLASS_INDEX_NODE, TREE_ALLOCATION)),
            NodeId::InTree(TreeName::Accounting, _) => Some((CLASS_INDEX_NODE, TREE_ACCOUNTING)),
            NodeId::InodeLeafContainer => Some((CLASS_PACKED, TREE_INODE)),
            NodeId::InodeRoot => Some((CLASS_INDEX_NODE, TREE_INODE)),
            NodeId::InTree(TreeName::Mapping, _) | NodeId::TreeTable => None,
        }
    }
    /// D16（发布语义） 已定项 9：空发布连带树表单元；根住树表的树换了根，树表单元就 COW。
    fn root_lives_in_tree_table(self) -> bool {
        match self {
            NodeId::InTree(TreeName::Extent | TreeName::Allocation | TreeName::Accounting, _) | NodeId::InodeLeafContainer | NodeId::InodeRoot => true,
            NodeId::InTree(TreeName::Mapping, _) | NodeId::TreeTable => false,
        }
    }
    fn span_slots(self) -> u64 {
        match self {
            NodeId::InodeLeafContainer => 2,
            NodeId::InTree(..) | NodeId::InodeRoot | NodeId::TreeTable => 1,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct NodeInfo { slot: u64, span: u64, mapping_key: Option<Key> }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlotState { Unused, Live, FreedWaiting, FreedReusable }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Operation { Append(u32), OverwriteRandom(u32), Empty }

#[derive(Clone, Copy, Debug)]
struct Config { split_policy: SplitPolicy, reuse_window: u64, allocation_internal_capacity: usize, mapping_internal_capacity: usize, disable_tree_table: bool }

#[derive(Clone, Copy, Debug, Default)]
struct PublishReport {
    meta_blocks: u64,
    allocation_dirty_leaves: u64,
    allocation_dirty_internal: u64,
    mapping_dirty: u64,
    accounting_dirty: u64,
    tree_table: u64,
    other_dirty: u64,
    splits_in_fixpoint: u64,
    allocation_levels_before: usize,
    mapping_levels_before: usize,
    allocation_levels_after: usize,
    mapping_levels_after: usize,
    fallbacks: u64,
    segments_opened: u64,
}

struct World {
    config: Config,
    extent: BPlusTree,
    allocation: BPlusTree,
    accounting: BPlusTree,
    mapping: BPlusTree,
    info: BTreeMap<NodeId, NodeInfo>,
    slot_state: Vec<SlotState>,
    has_record: Vec<bool>,
    unavailable_per_segment: Vec<u32>,
    empty_segments: BTreeSet<u64>,
    allocatable: BTreeSet<u64>,
    reuse_queue: VecDeque<(u64, u64)>,
    open_segment: Option<u64>,
    bump: u64,
    txg: u64,
    file_units: Vec<(u64, Key)>,
    write_order: u64,
    random_state: u64,
    fallbacks: u64,
    segments_opened: u64,
    /// 追踪开着时，固定点里每个脏节点的旧落点与两块盘上旧记录所在的叶记进这里（只用于解释一次发布为什么多出那几块）。
    trace: Option<Vec<String>>,
}

fn index_of(slot: u64) -> usize { usize::try_from(slot - UNIT_AREA_START_SLOT).expect("槽在单元区") }
fn segment_of(slot: u64) -> u64 { (slot - UNIT_AREA_START_SLOT) / SEGMENT_SLOTS }

impl World {
    fn after_first_transaction(config: Config) -> Self {
        let segments = UNIT_AREA_SLOTS / SEGMENT_SLOTS;
        let mut world = World {
            config,
            extent: BPlusTree::new(leaf_capacity(24, 112), internal_capacity(24), config.split_policy),
            allocation: BPlusTree::new(leaf_capacity(10, 20), config.allocation_internal_capacity, config.split_policy),
            accounting: BPlusTree::new(leaf_capacity(22, 34), internal_capacity(22), config.split_policy),
            mapping: BPlusTree::new(leaf_capacity(27, 55), config.mapping_internal_capacity, config.split_policy),
            info: BTreeMap::new(),
            slot_state: vec![SlotState::Unused; usize::try_from(UNIT_AREA_SLOTS).expect("槽数")],
            has_record: vec![false; usize::try_from(UNIT_AREA_SLOTS).expect("槽数")],
            unavailable_per_segment: vec![0; usize::try_from(segments).expect("段数")],
            empty_segments: (0..segments).collect(),
            allocatable: (UNIT_AREA_START_SLOT..UNIT_AREA_START_SLOT + UNIT_AREA_SLOTS).collect(),
            reuse_queue: VecDeque::new(),
            open_segment: Some(50240),
            bump: 50249,
            txg: 3,
            file_units: Vec::new(),
            write_order: 1,
            random_state: 0x9e37_79b9_7f4a_7c15,
            fallbacks: 0,
            segments_opened: 0,
            trace: None,
        };
        for (slot, span) in [(50176u64, 2u64), (50178, 1), (50180, 2), (50240, 1), (50242, 2), (50244, 1), (50245, 1), (50246, 1), (50247, 1), (50248, 1)] {
            world.take_slots(slot, span);
            for device in 0..DEVICES { world.allocation.insert(allocation_key(device, slot)); }
            world.has_record[index_of(slot)] = true;
        }
        // 开放段 [50240, 50304) 里剩下的槽只归 bump，不当「全空段」再开。
        world.empty_segments.remove(&segment_of(50240));
        world.file_units.push((50180, mapping_key(CLASS_DATA, TREE_EXTENT, 3, 1)));
        world.extent.insert(extent_key(0));
        for row in 0..15u128 { world.accounting.insert(row); }
        let nodes = [
            (NodeId::InTree(TreeName::Extent, 0), 50240, 1, Some(mapping_key(CLASS_INDEX_NODE, TREE_EXTENT, 3, 0))),
            (NodeId::InodeLeafContainer, 50242, 2, Some(mapping_key(CLASS_PACKED, TREE_INODE, 3, 0))),
            (NodeId::InodeRoot, 50244, 1, Some(mapping_key(CLASS_INDEX_NODE, TREE_INODE, 3, 1))),
            (NodeId::InTree(TreeName::Allocation, 0), 50245, 1, Some(mapping_key(CLASS_INDEX_NODE, TREE_ALLOCATION, 3, 0))),
            (NodeId::InTree(TreeName::Accounting, 0), 50246, 1, Some(mapping_key(CLASS_INDEX_NODE, TREE_ACCOUNTING, 3, 0))),
            (NodeId::InTree(TreeName::Mapping, 0), 50247, 1, None),
            (NodeId::TreeTable, 50248, 1, None),
        ];
        world.mapping.insert(world.file_units[0].1);
        for (id, slot, span, key) in nodes {
            if let Some(key) = key { world.mapping.insert(key); }
            world.info.insert(id, NodeInfo { slot, span, mapping_key: key });
        }
        assert_eq!(world.allocation.check_invariants(), 20, "字节表五：20 条分配记录");
        assert_eq!(world.mapping.check_invariants(), 6, "字节表三·二：6 条映射条目");
        world
    }

    fn take_slots(&mut self, slot: u64, span: u64) {
        for offset in 0..span {
            let current = slot + offset;
            let index = index_of(current);
            let state = self.slot_state[index];
            assert!(matches!(state, SlotState::Unused | SlotState::FreedReusable), "槽 {current} 不可分配：{state:?}");
            self.slot_state[index] = SlotState::Live;
            self.allocatable.remove(&current);
            let segment = segment_of(current);
            let counter = &mut self.unavailable_per_segment[usize::try_from(segment).expect("段号")];
            if *counter == 0 { self.empty_segments.remove(&segment); }
            *counter += 1;
        }
    }

    fn release_slots(&mut self, slot: u64, span: u64) {
        for offset in 0..span {
            let index = index_of(slot + offset);
            assert_eq!(self.slot_state[index], SlotState::Live, "释放的槽必须活着");
            self.slot_state[index] = SlotState::FreedWaiting;
            self.reuse_queue.push_back((self.txg + self.config.reuse_window, slot + offset));
        }
    }

    fn process_reuse(&mut self) {
        while let Some((due, slot)) = self.reuse_queue.front().copied() {
            if due > self.txg { break; }
            self.reuse_queue.pop_front();
            let index = index_of(slot);
            assert_eq!(self.slot_state[index], SlotState::FreedWaiting);
            self.slot_state[index] = SlotState::FreedReusable;
            self.allocatable.insert(slot);
            let segment = segment_of(slot);
            let counter = &mut self.unavailable_per_segment[usize::try_from(segment).expect("段号")];
            *counter -= 1;
            if *counter == 0 && self.open_segment.is_none_or(|open| segment_of(open) != segment) { self.empty_segments.insert(segment); }
        }
    }

    fn in_open_segment(&self, slot: u64) -> bool {
        self.open_segment.is_some_and(|open| slot >= open && slot < open + SEGMENT_SLOTS)
    }

    /// 用户数据：最低的偶数槽、两槽都可分配、不在开放段（D3（空间分配） 已定项 8 / 已定项 10 ②③）。
    fn allocate_user_data(&mut self) -> u64 {
        let chosen = self.allocatable.iter().copied()
            .find(|slot| slot % 2 == 0 && !self.in_open_segment(*slot) && self.allocatable.contains(&(slot + 1)))
            .expect("镜像写满了：没有偶数空槽对");
        self.take_slots(chosen, 2);
        chosen
    }

    /// 提交内生块：开放段 bump，容器 32768 对齐；段满开最低全空段；没有全空段回落到最低可分配槽（D3（空间分配） 已定项 8 第 2 条）。
    fn allocate_commit_generated(&mut self, span: u64) -> u64 {
        loop {
            if self.open_segment.is_none() {
                match self.empty_segments.iter().next().copied() {
                    Some(segment) => {
                        self.empty_segments.remove(&segment);
                        let start = UNIT_AREA_START_SLOT + segment * SEGMENT_SLOTS;
                        self.open_segment = Some(start);
                        self.bump = start;
                        self.segments_opened += 1;
                    }
                    None => {
                        self.fallbacks += 1;
                        let chosen = self.allocatable.iter().copied()
                            .find(|slot| span == 1 || (slot % 2 == 0 && self.allocatable.contains(&(slot + 1))))
                            .expect("镜像写满了：回落也找不到");
                        self.take_slots(chosen, span);
                        return chosen;
                    }
                }
            }
            let open = self.open_segment.expect("刚开");
            let mut start = self.bump;
            if span == 2 && start % 2 == 1 { start += 1; }
            if start + span > open + SEGMENT_SLOTS {
                self.open_segment = None;
                continue;
            }
            self.bump = start + span;
            self.take_slots(start, span);
            return start;
        }
    }

    fn record_allocation(&mut self, dirty: &mut BTreeSet<NodeId>, slot: u64) {
        let index = index_of(slot);
        for device in 0..DEVICES {
            let key = allocation_key(device, slot);
            let touched = if self.has_record[index] { self.allocation.touch(key) } else { self.allocation.insert(key) };
            for node in touched { dirty.insert(NodeId::InTree(TreeName::Allocation, node)); }
        }
        self.has_record[index] = true;
    }

    fn record_release(&mut self, dirty: &mut BTreeSet<NodeId>, slot: u64) {
        for device in 0..DEVICES {
            for node in self.allocation.touch(allocation_key(device, slot)) { dirty.insert(NodeId::InTree(TreeName::Allocation, node)); }
        }
    }

    fn next_random(&mut self) -> u64 {
        self.random_state = self.random_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        self.random_state >> 33
    }

    fn unavailable_slots(&self) -> u64 {
        UNIT_AREA_SLOTS - u64::try_from(self.allocatable.len()).expect("槽数")
    }

    fn publish(&mut self, operation: Operation) -> PublishReport {
        self.txg += 1;
        self.process_reuse();
        let txg = self.txg;
        let allocation_levels_before = self.allocation.levels;
        let mapping_levels_before = self.mapping.levels;
        let fallbacks_before = self.fallbacks;
        let segments_before = self.segments_opened;
        let mut dirty: BTreeSet<NodeId> = BTreeSet::new();
        let mut inode_dirty = false;
        match operation {
            Operation::Append(units) => {
                for _ in 0..units {
                    let slot = self.allocate_user_data();
                    self.record_allocation(&mut dirty, slot);
                    self.write_order += 1;
                    let key = mapping_key(CLASS_DATA, TREE_EXTENT, txg, self.write_order);
                    for node in self.mapping.insert(key) { dirty.insert(NodeId::InTree(TreeName::Mapping, node)); }
                    let offset = u64::try_from(self.file_units.len()).expect("偏移");
                    for node in self.extent.insert(extent_key(offset)) { dirty.insert(NodeId::InTree(TreeName::Extent, node)); }
                    self.file_units.push((slot, key));
                }
                inode_dirty = true;
            }
            Operation::OverwriteRandom(units) => {
                for _ in 0..units {
                    let position = usize::try_from(self.next_random() % u64::try_from(self.file_units.len()).expect("单元数")).expect("下标");
                    let slot = self.allocate_user_data();
                    self.record_allocation(&mut dirty, slot);
                    self.write_order += 1;
                    let key = mapping_key(CLASS_DATA, TREE_EXTENT, txg, self.write_order);
                    for node in self.mapping.insert(key) { dirty.insert(NodeId::InTree(TreeName::Mapping, node)); }
                    let (old_slot, old_key) = self.file_units[position];
                    self.release_slots(old_slot, 2);
                    self.record_release(&mut dirty, old_slot);
                    for node in self.mapping.remove(old_key) { dirty.insert(NodeId::InTree(TreeName::Mapping, node)); }
                    let offset = u64::try_from(position).expect("偏移");
                    for node in self.extent.touch(extent_key(offset)) { dirty.insert(NodeId::InTree(TreeName::Extent, node)); }
                    self.file_units[position] = (slot, key);
                }
                inode_dirty = true;
            }
            Operation::Empty => {}
        }
        if inode_dirty { dirty.insert(NodeId::InodeLeafContainer); dirty.insert(NodeId::InodeRoot); }
        // D16（发布语义） 已定项 9：记账树存在时每次发布按「每行每发布重写」重写记账行（375 条住一个叶，见 self_tests）。
        for node in self.accounting.touch(0) { dirty.insert(NodeId::InTree(TreeName::Accounting, node)); }
        let splits_before_fixpoint = self.allocation.splits + self.mapping.splits;

        let mut allocated: BTreeSet<NodeId> = BTreeSet::new();
        let mut birth_sequences: BTreeMap<u128, u64> = BTreeMap::new();
        let mut report = PublishReport { allocation_levels_before, mapping_levels_before, ..PublishReport::default() };
        while let Some(id) = dirty.iter().find(|candidate| !allocated.contains(candidate)).copied() {
            allocated.insert(id);
            let span = id.span_slots();
            let new_slot = self.allocate_commit_generated(span);
            if self.trace.is_some() {
                let old = self.info.get(&id).copied();
                let leaves = |slot: u64, tree: &BPlusTree| (0..DEVICES).map(|device| *tree.path_to_leaf(allocation_key(device, slot)).last().expect("路径非空")).collect::<Vec<_>>();
                let line = format!(
                    "node={id:?} new_slot={new_slot} new_slot_leaves={:?} old_slot={} old_slot_segment={} old_slot_leaves={:?}",
                    leaves(new_slot, &self.allocation),
                    old.map_or_else(|| "none".to_string(), |info| info.slot.to_string()),
                    old.map_or_else(|| "none".to_string(), |info| segment_of(info.slot).to_string()),
                    old.map(|info| leaves(info.slot, &self.allocation))
                );
                self.trace.as_mut().expect("开着").push(line);
            }
            report.meta_blocks += 1;
            match id {
                NodeId::InTree(TreeName::Allocation, node) => {
                    if self.allocation.nodes[node].is_leaf { report.allocation_dirty_leaves += 1 } else { report.allocation_dirty_internal += 1 }
                }
                NodeId::InTree(TreeName::Mapping, _) => report.mapping_dirty += 1,
                NodeId::InTree(TreeName::Accounting, _) => report.accounting_dirty += 1,
                NodeId::TreeTable => report.tree_table += 1,
                NodeId::InTree(TreeName::Extent, _) | NodeId::InodeLeafContainer | NodeId::InodeRoot => report.other_dirty += 1,
            }
            self.record_allocation(&mut dirty, new_slot);
            let new_key = id.mapping_identity().map(|(class, tree)| {
                let sequence = birth_sequences.entry(tree).or_insert(0);
                let key = mapping_key(class, tree, txg, *sequence);
                *sequence += 1;
                key
            });
            if let Some(key) = new_key { for node in self.mapping.insert(key) { dirty.insert(NodeId::InTree(TreeName::Mapping, node)); } }
            if let Some(old) = self.info.get(&id).copied() {
                self.release_slots(old.slot, old.span);
                self.record_release(&mut dirty, old.slot);
                if let Some(old_key) = old.mapping_key { for node in self.mapping.remove(old_key) { dirty.insert(NodeId::InTree(TreeName::Mapping, node)); } }
            }
            self.info.insert(id, NodeInfo { slot: new_slot, span, mapping_key: new_key });
            if id.root_lives_in_tree_table() && !self.config.disable_tree_table { dirty.insert(NodeId::TreeTable); }
        }
        report.splits_in_fixpoint = self.allocation.splits + self.mapping.splits - splits_before_fixpoint;
        report.allocation_levels_after = self.allocation.levels;
        report.mapping_levels_after = self.mapping.levels;
        report.fallbacks = self.fallbacks - fallbacks_before;
        report.segments_opened = self.segments_opened - segments_before;
        report
    }
}

/// ckpt_cost 形态句的两种读法（`28-挂载期承诺量.md:100`）：E148 的记账 2 个节点；按字面「记账树每发布的节点数」取第一版实际的 1。
fn formula_e148(report: &PublishReport) -> i64 { i64::try_from(report.allocation_levels_before + report.mapping_levels_before + 2).expect("块") }
fn formula_literal(report: &PublishReport) -> i64 { i64::try_from(report.allocation_levels_before + report.mapping_levels_before + 1).expect("块") }
/// 攻方腿自己提的收严（只算线索，被攻过零轮）：树表 1 + 记账 1 + 分配记录树两块盘各一条路径（共用根）+ 映射树一条路径。
fn formula_two_paths(report: &PublishReport) -> i64 { i64::try_from(2 + (2 * report.allocation_levels_before - 1) + report.mapping_levels_before).expect("块") }

fn default_config() -> Config {
    Config { split_policy: SplitPolicy::Half, reuse_window: 25, allocation_internal_capacity: internal_capacity(10), mapping_internal_capacity: internal_capacity(27), disable_tree_table: false }
}

fn self_tests() {
    assert_eq!(leaf_capacity(10, 20), 812, "分配记录叶");
    assert_eq!(internal_capacity(10), 169, "分配记录内部（假设）");
    assert_eq!(leaf_capacity(27, 55), 294, "映射叶");
    assert_eq!(internal_capacity(27), 143, "映射内部（假设）");
    assert!(ACCOUNTING_ROWS_KEPT <= leaf_capacity(22, 34), "记账 375 条住一个叶（叶 477）");
    // 第一个事务之后一次空发布：记账 1 + 分配记录 1 + 映射 1 + 树表 1 = 4（E148 第一个事务规模也是 4，E148 的 4 = 1 + 1 + 记账 2）。
    let mut world = World::after_first_transaction(default_config());
    let empty = world.publish(Operation::Empty);
    assert_eq!((empty.meta_blocks, empty.accounting_dirty, empty.allocation_dirty_leaves, empty.mapping_dirty, empty.tree_table), (4, 1, 1, 1, 1));
    assert_eq!(formula_e148(&empty), 4);
    assert_eq!(formula_literal(&empty), 3);
    // 第一个事务之后一次覆盖写：六个提交内生块 + 树表单元 = 7 块（里程碑二步 1 那一句）。
    let mut world = World::after_first_transaction(default_config());
    let overwrite = world.publish(Operation::OverwriteRandom(1));
    assert_eq!(overwrite.meta_blocks, 7);
    // 手算锚点：追加 896 个单元，每块盘的分配记录 > 896 > 812（一个叶装 812 条）⇒ 盘 0 最新那条记录与盘 1 最新那条之间
    // 隔着盘 1 在它之下的全部记录（> 812 条），装不进同一个叶；两块盘合计 < 812 × 169 ⇒ 树 2 层。
    // 之后的空发布：分配记录树脏 根 + 盘 0 的叶 + 盘 1 的叶 = 3（公式按层数算 2）。
    // （448 个单元时每块盘约 560 条，盘 0 的最新记录与盘 1 的全部记录还装得进一个叶，那一档不分叉——第一次写这条锚点时按合计 > 812 算错了。）
    let mut world = World::after_first_transaction(default_config());
    for _ in 0..14 { world.publish(Operation::Append(64)); }
    for _ in 0..3 { world.publish(Operation::Empty); }
    let steady = world.publish(Operation::Empty);
    assert_eq!(world.allocation.levels, 2);
    assert_eq!((steady.allocation_dirty_internal, steady.allocation_dirty_leaves), (1, 2), "两块盘的记录各在一个叶上");
    // 映射树也是 2 层（896 条数据条目 > 一个叶的 294 条）；空发布只动 (类 2, 树 13) 与 (类 2, 树 14) 两段的末尾，两段紧挨着 ⇒ 一条路径 2 块。
    assert_eq!(world.mapping.levels, 2);
    assert_eq!(steady.mapping_dirty, 2, "映射树一条根到叶的路径");
    assert_eq!(steady.meta_blocks, 1 + 3 + 2 + 1, "记账 1 + 分配记录 3 + 映射 2 + 树表 1");
    assert_eq!(formula_e148(&steady), 2 + 2 + 2, "E148 读法：分配记录 2 层 + 映射 2 层 + 记账 2");
    assert_eq!(formula_two_paths(&steady), 7, "收严读法：树表 1 + 记账 1 + 分配记录 3 + 映射 2");
    println!("V3RESULT name=selftest passed=15 steady_after_896_units={}", report_text(&steady));
}

fn report_text(report: &PublishReport) -> String {
    format!(
        "meta_blocks={} acct={} alloc_internal={} alloc_leaves={} map={} tree_table={} other={} splits={} levels_before=alloc{}:map{} levels_after=alloc{}:map{} fallbacks={} segments_opened={} formula_e148={} formula_literal={}",
        report.meta_blocks, report.accounting_dirty, report.allocation_dirty_internal, report.allocation_dirty_leaves, report.mapping_dirty, report.tree_table, report.other_dirty,
        report.splits_in_fixpoint, report.allocation_levels_before, report.mapping_levels_before, report.allocation_levels_after, report.mapping_levels_after,
        report.fallbacks, report.segments_opened, formula_e148(report), formula_literal(report)
    )
}

fn histogram_text(histogram: &BTreeMap<i64, u64>) -> String {
    histogram.iter().map(|(deficit, count)| format!("{deficit}:{count}")).collect::<Vec<_>>().join(",")
}

fn run(name: &str, config: Config, fill_percent: u64, empty_publishes: u64) {
    let mut world = World::after_first_transaction(config);
    let target = UNIT_AREA_SLOTS * fill_percent / 100;
    let mut write_publishes = 0u64;
    while world.unavailable_slots() < target {
        write_publishes += 1;
        if write_publishes % 4 == 0 { world.publish(Operation::OverwriteRandom(32)); } else { world.publish(Operation::Append(64)); }
    }
    let mut deficits_e148: BTreeMap<i64, u64> = BTreeMap::new();
    let mut deficits_literal: BTreeMap<i64, u64> = BTreeMap::new();
    let mut deficits_two_paths: BTreeMap<i64, u64> = BTreeMap::new();
    let mut worst_two_paths_without_new_segment = i64::MIN;
    let mut worst: Option<PublishReport> = None;
    let mut first: Option<PublishReport> = None;
    let mut max_alloc_leaves = 0u64;
    let mut max_alloc_internal = 0u64;
    let mut max_map = 0u64;
    let mut split_publishes = 0u64;
    let mut fallback_publishes = 0u64;
    let mut segment_publishes = 0u64;
    for index in 0..empty_publishes {
        let report = world.publish(Operation::Empty);
        let actual = i64::try_from(report.meta_blocks).expect("块");
        *deficits_e148.entry(actual - formula_e148(&report)).or_insert(0) += 1;
        *deficits_literal.entry(actual - formula_literal(&report)).or_insert(0) += 1;
        *deficits_two_paths.entry(actual - formula_two_paths(&report)).or_insert(0) += 1;
        if report.segments_opened == 0 { worst_two_paths_without_new_segment = worst_two_paths_without_new_segment.max(actual - formula_two_paths(&report)); }
        if worst.as_ref().is_none_or(|best| report.meta_blocks > best.meta_blocks) { worst = Some(report); }
        if index == 0 { first = Some(report); }
        max_alloc_leaves = max_alloc_leaves.max(report.allocation_dirty_leaves);
        max_alloc_internal = max_alloc_internal.max(report.allocation_dirty_internal);
        max_map = max_map.max(report.mapping_dirty);
        split_publishes += u64::from(report.splits_in_fixpoint > 0);
        fallback_publishes += u64::from(report.fallbacks > 0);
        segment_publishes += u64::from(report.segments_opened > 0);
    }
    let records = world.allocation.check_invariants();
    let entries = world.mapping.check_invariants();
    world.extent.check_invariants();
    println!(
        "V3RESULT name=fill workload={name} fill_percent={fill_percent} write_publishes={write_publishes} unavailable_slots={} allocation_records={records} mapping_entries={entries} levels=alloc{}:map{}:extent{} policy={:?} reuse_window={} alloc_internal_cap={} map_internal_cap={}",
        world.unavailable_slots(), world.allocation.levels, world.mapping.levels, world.extent.levels, config.split_policy, config.reuse_window,
        config.allocation_internal_capacity, config.mapping_internal_capacity
    );
    println!(
        "V3RESULT name=empty_publishes workload={name} fill_percent={fill_percent} count={empty_publishes} deficit_vs_e148={} deficit_vs_literal={} deficit_vs_two_paths={} worst_two_paths_deficit_without_new_segment={worst_two_paths_without_new_segment} max_alloc_internal={max_alloc_internal} max_alloc_leaves={max_alloc_leaves} max_map={max_map} publishes_with_fixpoint_split={split_publishes} publishes_with_fallback={fallback_publishes} publishes_opening_segment={segment_publishes}",
        histogram_text(&deficits_e148), histogram_text(&deficits_literal), histogram_text(&deficits_two_paths)
    );
    println!("V3RESULT name=first_empty workload={name} fill_percent={fill_percent} {}", report_text(&first.expect("至少一次")));
    println!("V3RESULT name=worst_empty workload={name} fill_percent={fill_percent} {}", report_text(&worst.expect("至少一次")));
}

/// 85% 那一档里不开新段、却比两条路径的收严读法多得最多的那次空发布，逐个脏节点打出旧落点在哪个叶上。
fn trace_worst_without_new_segment(config: Config, fill_percent: u64, empty_publishes: u64) {
    let fill = |world: &mut World| {
        let target = UNIT_AREA_SLOTS * fill_percent / 100;
        let mut write_publishes = 0u64;
        while world.unavailable_slots() < target {
            write_publishes += 1;
            if write_publishes % 4 == 0 { world.publish(Operation::OverwriteRandom(32)); } else { world.publish(Operation::Append(64)); }
        }
    };
    let mut world = World::after_first_transaction(config);
    fill(&mut world);
    let mut chosen: Option<(u64, i64)> = None;
    for index in 0..empty_publishes {
        let report = world.publish(Operation::Empty);
        let deficit = i64::try_from(report.meta_blocks).expect("块") - formula_two_paths(&report);
        if report.segments_opened == 0 && chosen.is_none_or(|(_, best)| deficit > best) { chosen = Some((index, deficit)); }
    }
    let (chosen_index, deficit) = chosen.expect("至少一次");
    let mut world = World::after_first_transaction(config);
    fill(&mut world);
    for index in 0..=chosen_index {
        if index + 1 >= chosen_index { world.trace = Some(Vec::new()); }
        let open_before = world.open_segment;
        let bump_before = world.bump;
        let report = world.publish(Operation::Empty);
        if let Some(lines) = world.trace.take() {
            println!("V3TRACE publish_index={index} open_segment_start={open_before:?} bump_before={bump_before} {}", report_text(&report));
            for line in lines { println!("V3TRACE   {line}"); }
        }
    }
    println!("V3RESULT name=trace_chosen fill_percent={fill_percent} publish_index={chosen_index} deficit_vs_two_paths={deficit}");
}

fn main() {
    self_tests();
    let base = default_config();
    for fill in [0u64, 1, 10, 35, 60, 85] {
        run("half_reuse25", base, fill, 400);
    }
    for fill in [35u64, 85] {
        run("right_end_split_reuse25", Config { split_policy: SplitPolicy::RightEndAppend, ..base }, fill, 400);
        run("half_reuse25_internal64", Config { allocation_internal_capacity: 64, mapping_internal_capacity: 64, ..base }, fill, 400);
        run("half_reuse25_internal512", Config { allocation_internal_capacity: 512, mapping_internal_capacity: 512, ..base }, fill, 400);
        run("half_reuse200", Config { reuse_window: 200, ..base }, fill, 400);
    }
    trace_worst_without_new_segment(base, 85, 400);
    println!("V3RESULT name=done");
}
