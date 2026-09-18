//! C363（现算保留池时树高从哪读没有条款）第一轮攻方腿：问二的固定点模型。只用 std。
//! 从第一个事务之后的盘面（layout/01-first-txn.md 零 / 五 / 七节，crates/ 今天的落点与宽度）起，按固定脚本逐次发布，
//! 每次发布跑完整固定点：脏节点各要一个新落点（两盘同号）→ 分配记录树每盘各插一条、映射树插一条（映射树与树表豁免）→
//! 被 COW 的旧版本改写分配记录、删映射条目 → 这些插入 / 删除再脏节点，直到没有未分配的脏节点。
//! 节点真分裂（B+ 树），不合并（合并没有条款）；释放的槽不回收（defer，给的是下界）。
//! 报：固定点实际分配的元数据块数 / 槽数，与三条臂在准入那一刻扣的量。

use std::collections::{BTreeMap, BTreeSet};

type Key = u128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SplitPolicy {
    Half,
    RightEndAppend,
}

#[derive(Clone, Debug)]
struct TreeNode {
    is_leaf: bool,
    /// 叶：条目 key；内部：分隔 key，keys[i] = children[i + 1] 子树里最小的 key。
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

/// 一次插入 / 删除 / 改写动到的节点：根到叶的路径 + 分裂新建的节点。
#[derive(Default, Debug)]
struct Touched {
    nodes: Vec<usize>,
}

impl BPlusTree {
    fn new(leaf_capacity: usize, internal_capacity: usize, split_policy: SplitPolicy) -> Self {
        Self {
            leaf_capacity,
            internal_capacity,
            split_policy,
            nodes: vec![TreeNode { is_leaf: true, keys: Vec::new(), children: Vec::new() }],
            root: 0,
            levels: 1,
            splits: 0,
        }
    }

    fn path_to_leaf(&self, key: Key) -> Vec<usize> {
        let mut path = vec![self.root];
        let mut current = self.root;
        while !self.nodes[current].is_leaf {
            let node = &self.nodes[current];
            let child_position = node.keys.partition_point(|separator| *separator <= key);
            current = node.children[child_position];
            path.push(current);
        }
        path
    }

    fn touch(&self, key: Key) -> Touched {
        Touched { nodes: self.path_to_leaf(key) }
    }

    fn remove(&mut self, key: Key) -> Touched {
        let path = self.path_to_leaf(key);
        let leaf = *path.last().expect("路径至少有根");
        let position = self.nodes[leaf].keys.binary_search(&key).expect("删的 key 必须在树里");
        self.nodes[leaf].keys.remove(position);
        Touched { nodes: path }
    }

    fn insert(&mut self, key: Key) -> Touched {
        let path = self.path_to_leaf(key);
        let leaf = *path.last().expect("路径至少有根");
        let position = match self.nodes[leaf].keys.binary_search(&key) {
            Ok(_) => panic!("key 重复插入：{key}"),
            Err(position) => position,
        };
        self.nodes[leaf].keys.insert(position, key);
        let mut touched = path.clone();
        let mut level_from_bottom = path.len();
        while level_from_bottom > 0 {
            level_from_bottom -= 1;
            let node_index = path[level_from_bottom];
            let over = if self.nodes[node_index].is_leaf {
                self.nodes[node_index].keys.len() > self.leaf_capacity
            } else {
                self.nodes[node_index].children.len() > self.internal_capacity
            };
            if !over {
                break;
            }
            let (separator, right_index) = self.split(node_index, key);
            touched.push(right_index);
            if level_from_bottom == 0 {
                let new_root = self.nodes.len();
                self.nodes.push(TreeNode { is_leaf: false, keys: vec![separator], children: vec![node_index, right_index] });
                self.root = new_root;
                self.levels += 1;
                touched.push(new_root);
            } else {
                let parent = path[level_from_bottom - 1];
                let child_position = self.nodes[parent].children.iter().position(|child| *child == node_index).expect("父节点里有它");
                self.nodes[parent].keys.insert(child_position, separator);
                self.nodes[parent].children.insert(child_position + 1, right_index);
            }
        }
        Touched { nodes: touched }
    }

    /// 分裂 node_index，返回 (提到父节点的分隔 key, 新建的右半)。RightEndAppend：插入落在最右端时左半保留全部、右半只拿最后一个。
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
            let inserted_into_last_child = node.keys.last().map_or(true, |last| inserted_key >= *last);
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

    /// 自检：叶同深、key 有序、分隔 key 与子树一致、容量不超；返回 key 总数。
    fn check_invariants(&self) -> usize {
        fn walk(tree: &BPlusTree, index: usize, depth: usize, low: Option<Key>, high: Option<Key>) -> usize {
            let node = &tree.nodes[index];
            assert!(node.keys.windows(2).all(|pair| pair[0] < pair[1]), "key 无序");
            if let Some(first) = node.keys.first() { assert!(low.map_or(true, |bound| *first >= bound), "key 低于区间"); }
            if let Some(last) = node.keys.last() { assert!(high.map_or(true, |bound| *last < bound), "key 高于区间"); }
            if node.is_leaf {
                assert_eq!(depth, tree.levels, "叶不同深");
                assert!(node.keys.len() <= tree.leaf_capacity, "叶超容量");
                return node.keys.len();
            }
            assert!(node.children.len() <= tree.internal_capacity, "内部节点超容量");
            assert_eq!(node.keys.len() + 1, node.children.len(), "分隔 key 数不对");
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

// ---- 盘面：crates/ 今天的宽度（singlefs-format）与第一个事务之后的落点 ----
const UNIT_AREA_START_SLOT: u64 = 50176;
const UNIT_AREA_SLOTS: u64 = 1 << 22;
const SEGMENT_SLOTS: u64 = 64;
const NODE_BYTES: usize = 16384;
fn index_node_header_bytes(key_width: usize) -> usize { 86 + 2 * key_width + 29 }
const NODE_POINTER_BYTES: usize = 86;
/// 叶容量按 crates/ 的条目宽；内部节点条目格式只有 inode 树有条款（records/2026-09-13-总审核.md「D8-D11 发现 16」），
/// 这里假设内部条目 = key + 86 字节子指针，与 inode 树内部条目去掉身份引用同形——假设，不是条款。
fn leaf_capacity(key_width: usize, entry_width: usize) -> usize { (NODE_BYTES - index_node_header_bytes(key_width)) / entry_width }
fn internal_capacity(key_width: usize) -> usize { (NODE_BYTES - index_node_header_bytes(key_width)) / (key_width + NODE_POINTER_BYTES) }

const TREE_EXTENT: u128 = 11;
const TREE_INODE: u128 = 12;
const TREE_ALLOCATION: u128 = 13;
const TREE_ACCOUNTING: u128 = 14;
const CLASS_DATA: u128 = 1;
const CLASS_INDEX_NODE: u128 = 2;
const CLASS_PACKED: u128 = 3;
const DEVICES: u128 = 2;

/// 映射 key 的比较序 = 类标签 → 出生树 → 出生 txg → 尾段（records.rs 的 mapping_key_sort_key）。
fn mapping_key(class: u128, tree: u128, txg: u64, tail: u64) -> Key { (class << 120) | (tree << 104) | (u128::from(txg) << 64) | u128::from(tail) }
/// 分配记录 key = (设备身份, 槽号)（allocator.rs 的 sort_key）。
fn allocation_key(device: u128, slot: u64) -> Key { (device << 64) | u128::from(slot) }
fn extent_key(inode: u64, offset_units: u64) -> Key { (u128::from(inode) << 64) | u128::from(offset_units) }

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TreeName { Extent, Allocation, Accounting, Mapping }

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum NodeId { InTree(TreeName, usize), InodeLeafContainer, InodeRoot, TreeTable }

impl NodeId {
    /// (类标签, 出生树)；映射树自己与树表豁免（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
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
    /// 根住树表的树：它的任何节点脏了，根就换了，树表单元跟着 COW（D16（发布语义） 已定项 9 那一行把树表单元列进空发布）。
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
struct NodeInfo { slot: u64, mapping_key: Option<Key> }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Levels { extent: usize, allocation: usize, accounting: usize, mapping: usize }
const INODE_TREE_LEVELS: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Operation { OverwriteFirstUnit, Append(u32), Empty }

struct World {
    extent: BPlusTree,
    allocation: BPlusTree,
    accounting: BPlusTree,
    mapping: BPlusTree,
    info: BTreeMap<NodeId, NodeInfo>,
    used: Vec<bool>,
    segment_used: Vec<u32>,
    open_segment: Option<u64>,
    bump: u64,
    lowest_candidate_pair: u64,
    lowest_candidate_segment: u64,
    txg: u64,
    file_slots: Vec<u64>,
    file_mapping_keys: Vec<Key>,
    write_order: u64,
}

#[derive(Clone, Debug)]
struct PublishReport {
    txg: u64,
    operation: Operation,
    meta_blocks: u64,
    meta_slots: u64,
    levels_before: Levels,
    levels_after_user_operations: Levels,
    levels_after: Levels,
    dirty_nodes_per_tree: [u64; 4],
    trees_dirtied: [bool; 4],
    inode_dirtied: bool,
    splits_in_user_operations: u64,
    splits_in_fixpoint: u64,
}

impl World {
    fn levels(&self) -> Levels {
        Levels { extent: self.extent.levels, allocation: self.allocation.levels, accounting: self.accounting.levels, mapping: self.mapping.levels }
    }
    fn total_splits(&self) -> u64 { self.extent.splits + self.allocation.splits + self.accounting.splits + self.mapping.splits }

    fn mark_used(&mut self, slot: u64, span: u64) {
        for offset in 0..span {
            let index = usize::try_from(slot + offset - UNIT_AREA_START_SLOT).expect("槽在单元区");
            assert!(!self.used[index], "槽 {} 已被占", slot + offset);
            self.used[index] = true;
            self.segment_used[index / 64] += 1;
        }
    }
    fn is_free(&self, slot: u64) -> bool { !self.used[usize::try_from(slot - UNIT_AREA_START_SLOT).expect("槽在单元区")] }

    /// 提交内生块：开放段 bump，容器 32768 对齐，段满开最低全空段（allocator.rs 的 allocate_commit_generated）。
    fn allocate_commit_generated(&mut self, span: u64) -> u64 {
        loop {
            if self.open_segment.is_none() {
                while self.segment_used[usize::try_from(self.lowest_candidate_segment).expect("段号")] != 0 { self.lowest_candidate_segment += 1; }
                let start = UNIT_AREA_START_SLOT + self.lowest_candidate_segment * SEGMENT_SLOTS;
                self.open_segment = Some(start);
                self.bump = start;
            }
            let open = self.open_segment.expect("刚开");
            let mut start = self.bump;
            if span == 2 && start % 2 == 1 { start += 1; }
            if start + span > open + SEGMENT_SLOTS {
                self.lowest_candidate_pair = self.lowest_candidate_pair.min(open);
                self.open_segment = None;
                continue;
            }
            self.bump = start + span;
            self.mark_used(start, span);
            return start;
        }
    }
    /// 用户数据：最低的偶数空槽对、不在开放段（allocator.rs 的 lowest_user_data_slot）。
    fn allocate_user_data(&mut self) -> u64 {
        let mut candidate = self.lowest_candidate_pair;
        loop {
            let in_open = self.open_segment.is_some_and(|open| candidate >= open && candidate < open + SEGMENT_SLOTS);
            if !in_open && self.is_free(candidate) && self.is_free(candidate + 1) { break; }
            candidate += 2;
        }
        self.lowest_candidate_pair = candidate;
        self.mark_used(candidate, 2);
        candidate
    }

    fn after_first_transaction(split_policy: SplitPolicy) -> Self {
        let mut world = World {
            extent: BPlusTree::new(leaf_capacity(24, 112), internal_capacity(24), split_policy),
            allocation: BPlusTree::new(leaf_capacity(10, 20), internal_capacity(10), split_policy),
            accounting: BPlusTree::new(leaf_capacity(22, 34), internal_capacity(22), split_policy),
            mapping: BPlusTree::new(leaf_capacity(27, 55), internal_capacity(27), split_policy),
            info: BTreeMap::new(),
            used: vec![false; usize::try_from(UNIT_AREA_SLOTS).expect("槽数")],
            segment_used: vec![0; usize::try_from(UNIT_AREA_SLOTS / SEGMENT_SLOTS).expect("段数")],
            open_segment: Some(50240), bump: 50249, lowest_candidate_pair: 50182, lowest_candidate_segment: 0,
            txg: 3, file_slots: vec![50180], file_mapping_keys: vec![mapping_key(CLASS_DATA, TREE_EXTENT, 3, 1)], write_order: 1,
        };
        for (slot, span) in [(50176, 2), (50178, 1), (50180, 2), (50240, 1), (50242, 2), (50244, 1), (50245, 1), (50246, 1), (50247, 1), (50248, 1)] {
            world.mark_used(slot, span);
            for device in 0..DEVICES { world.allocation.insert(allocation_key(device, slot)); }
        }
        world.extent.insert(extent_key(1, 0));
        for row in 0..15u128 { world.accounting.insert(row); }
        let nodes = [
            (NodeId::InTree(TreeName::Extent, 0), 50240, Some(mapping_key(CLASS_INDEX_NODE, TREE_EXTENT, 3, 0))),
            (NodeId::InodeLeafContainer, 50242, Some(mapping_key(CLASS_PACKED, TREE_INODE, 3, 0))),
            (NodeId::InodeRoot, 50244, Some(mapping_key(CLASS_INDEX_NODE, TREE_INODE, 3, 1))),
            (NodeId::InTree(TreeName::Allocation, 0), 50245, Some(mapping_key(CLASS_INDEX_NODE, TREE_ALLOCATION, 3, 0))),
            (NodeId::InTree(TreeName::Accounting, 0), 50246, Some(mapping_key(CLASS_INDEX_NODE, TREE_ACCOUNTING, 3, 0))),
            (NodeId::InTree(TreeName::Mapping, 0), 50247, None),
            (NodeId::TreeTable, 50248, None),
        ];
        world.mapping.insert(world.file_mapping_keys[0]);
        for (id, slot, key) in nodes {
            if let Some(key) = key { world.mapping.insert(key); }
            world.info.insert(id, NodeInfo { slot, mapping_key: key });
        }
        assert_eq!(world.allocation.check_invariants(), 20, "字节表五：20 条分配记录");
        assert_eq!(world.mapping.check_invariants(), 6, "字节表三·二：6 条映射条目");
        world
    }

    fn mark(dirty: &mut BTreeSet<NodeId>, name: TreeName, touched: Touched) {
        for index in touched.nodes { dirty.insert(NodeId::InTree(name, index)); }
    }

    fn publish(&mut self, operation: Operation) -> PublishReport {
        self.txg += 1;
        let txg = self.txg;
        let levels_before = self.levels();
        let splits_at_start = self.total_splits();
        let mut dirty: BTreeSet<NodeId> = BTreeSet::new();
        let mut birth_sequences: BTreeMap<u128, u64> = BTreeMap::new();
        let mut inode_dirtied = false;
        match operation {
            Operation::OverwriteFirstUnit => {
                let new_slot = self.allocate_user_data();
                for device in 0..DEVICES { let touched = self.allocation.insert(allocation_key(device, new_slot)); Self::mark(&mut dirty, TreeName::Allocation, touched); }
                self.write_order += 1;
                let new_key = mapping_key(CLASS_DATA, TREE_EXTENT, txg, self.write_order);
                let touched = self.mapping.insert(new_key); Self::mark(&mut dirty, TreeName::Mapping, touched);
                let (old_slot, old_key) = (self.file_slots[0], self.file_mapping_keys[0]);
                for device in 0..DEVICES { let touched = self.allocation.touch(allocation_key(device, old_slot)); Self::mark(&mut dirty, TreeName::Allocation, touched); }
                let touched = self.mapping.remove(old_key); Self::mark(&mut dirty, TreeName::Mapping, touched);
                let touched = self.extent.touch(extent_key(1, 0)); Self::mark(&mut dirty, TreeName::Extent, touched);
                self.file_slots[0] = new_slot;
                self.file_mapping_keys[0] = new_key;
                inode_dirtied = true;
            }
            Operation::Append(units) => {
                for _ in 0..units {
                    let new_slot = self.allocate_user_data();
                    for device in 0..DEVICES { let touched = self.allocation.insert(allocation_key(device, new_slot)); Self::mark(&mut dirty, TreeName::Allocation, touched); }
                    self.write_order += 1;
                    let new_key = mapping_key(CLASS_DATA, TREE_EXTENT, txg, self.write_order);
                    let touched = self.mapping.insert(new_key); Self::mark(&mut dirty, TreeName::Mapping, touched);
                    let offset = u64::try_from(self.file_slots.len()).expect("偏移");
                    let touched = self.extent.insert(extent_key(1, offset)); Self::mark(&mut dirty, TreeName::Extent, touched);
                    self.file_slots.push(new_slot);
                    self.file_mapping_keys.push(new_key);
                }
                inode_dirtied = true;
            }
            Operation::Empty => {}
        }
        if inode_dirtied { dirty.insert(NodeId::InodeLeafContainer); dirty.insert(NodeId::InodeRoot); }
        // D16（发布语义） 已定项 9：记账树存在时每次发布（含空发布）按「每行每发布重写」重写记账行。
        let touched = self.accounting.touch(0); Self::mark(&mut dirty, TreeName::Accounting, touched);
        let levels_after_user_operations = self.levels();
        let splits_after_user_operations = self.total_splits();

        let mut allocated: BTreeSet<NodeId> = BTreeSet::new();
        let (mut meta_blocks, mut meta_slots) = (0u64, 0u64);
        let mut dirty_nodes_per_tree = [0u64; 4];
        while let Some(id) = dirty.iter().find(|candidate| !allocated.contains(candidate)).copied() {
            allocated.insert(id);
            let span = id.span_slots();
            let new_slot = self.allocate_commit_generated(span);
            meta_blocks += 1;
            meta_slots += span;
            if let NodeId::InTree(name, _) = id { dirty_nodes_per_tree[name as usize] += 1; }
            for device in 0..DEVICES { let touched = self.allocation.insert(allocation_key(device, new_slot)); Self::mark(&mut dirty, TreeName::Allocation, touched); }
            let new_key = id.mapping_identity().map(|(class, tree)| {
                let sequence = birth_sequences.entry(tree).or_insert(0);
                let key = mapping_key(class, tree, txg, *sequence);
                *sequence += 1;
                key
            });
            if let Some(key) = new_key { let touched = self.mapping.insert(key); Self::mark(&mut dirty, TreeName::Mapping, touched); }
            if let Some(old) = self.info.get(&id).copied() {
                for device in 0..DEVICES { let touched = self.allocation.touch(allocation_key(device, old.slot)); Self::mark(&mut dirty, TreeName::Allocation, touched); }
                if let Some(old_key) = old.mapping_key { let touched = self.mapping.remove(old_key); Self::mark(&mut dirty, TreeName::Mapping, touched); }
            }
            self.info.insert(id, NodeInfo { slot: new_slot, mapping_key: new_key });
            if id.root_lives_in_tree_table() { dirty.insert(NodeId::TreeTable); }
        }
        let trees_dirtied = [TreeName::Extent, TreeName::Allocation, TreeName::Accounting, TreeName::Mapping]
            .map(|name| dirty.iter().any(|id| matches!(id, NodeId::InTree(dirty_name, _) if *dirty_name == name)));
        PublishReport {
            txg, operation, meta_blocks, meta_slots, levels_before, levels_after_user_operations, levels_after: self.levels(),
            dirty_nodes_per_tree, trees_dirtied, inode_dirtied,
            splits_in_user_operations: splits_after_user_operations - splits_at_start,
            splits_in_fixpoint: self.total_splits() - splits_after_user_operations,
        }
    }
}

// ---- 三条臂在准入那一刻扣的量（单位：16 KiB 块，与 D28（挂载期承诺量） 已定项 4 同） ----
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TreeSet { E148RecordTreesPlusTwoAccountingNodes, JiaEveryRootedTreePlusMapping }
const TREE_SETS: [TreeSet; 2] = [TreeSet::E148RecordTreesPlusTwoAccountingNodes, TreeSet::JiaEveryRootedTreePlusMapping];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm { JiaReadAtStartOfPreviousPublish, YiInMemoryTreesUpdatedAtPublish, YiInMemoryTreesUpdatedAtOperation, BingPublishedPlusOneLevelEach }
const ARMS: [Arm; 4] = [Arm::JiaReadAtStartOfPreviousPublish, Arm::YiInMemoryTreesUpdatedAtPublish, Arm::YiInMemoryTreesUpdatedAtOperation, Arm::BingPublishedPlusOneLevelEach];

fn formula(set: TreeSet, levels: Levels) -> u64 {
    let blocks = match set {
        TreeSet::E148RecordTreesPlusTwoAccountingNodes => levels.allocation + levels.mapping + 2,
        TreeSet::JiaEveryRootedTreePlusMapping => levels.extent + INODE_TREE_LEVELS + levels.allocation + levels.accounting + levels.mapping,
    };
    u64::try_from(blocks).expect("块数")
}
fn tree_count(set: TreeSet) -> u64 {
    match set { TreeSet::E148RecordTreesPlusTwoAccountingNodes => 2, TreeSet::JiaEveryRootedTreePlusMapping => 5 }
}
/// previous_start = 上一次发布开始时的层级（= 再上一次发布之后）；report.levels_before = 上一次发布之后。
fn arm_value(arm: Arm, set: TreeSet, previous_start: Levels, report: &PublishReport) -> u64 {
    match arm {
        Arm::JiaReadAtStartOfPreviousPublish => formula(set, previous_start),
        Arm::YiInMemoryTreesUpdatedAtPublish => formula(set, report.levels_before),
        Arm::YiInMemoryTreesUpdatedAtOperation => formula(set, report.levels_after_user_operations),
        Arm::BingPublishedPlusOneLevelEach => formula(set, report.levels_before) + tree_count(set),
    }
}

fn levels_text(levels: Levels) -> String {
    format!("extent{}:alloc{}:acct{}:map{}", levels.extent, levels.allocation, levels.accounting, levels.mapping)
}
fn report_text(report: &PublishReport) -> String {
    format!(
        "txg={} op={:?} meta_blocks={} meta_slots={} levels_before={} levels_after={} dirty_extent_alloc_acct_map={:?} inode_dirty={} splits_user={} splits_fixpoint={}",
        report.txg, report.operation, report.meta_blocks, report.meta_slots, levels_text(report.levels_before), levels_text(report.levels_after),
        report.dirty_nodes_per_tree, report.inode_dirtied, report.splits_in_user_operations, report.splits_in_fixpoint
    )
}

fn self_tests() {
    for policy in [SplitPolicy::Half, SplitPolicy::RightEndAppend] {
        let mut clustered = BPlusTree::new(812, 169, policy);
        for key in 0..200_000u128 { clustered.insert(key); }
        assert_eq!(clustered.check_invariants(), 200_000);
        let splits_before_probe = clustered.splits;
        let touched = clustered.insert(10_000_000);
        if clustered.splits == splits_before_probe {
            assert_eq!(touched.nodes.len(), clustered.levels, "不分裂的插入只动一条根到叶的路径");
        } else {
            assert!(touched.nodes.len() > clustered.levels, "分裂的插入多动新建的节点");
        }
        let mut scattered = BPlusTree::new(294, 143, policy);
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        for _ in 0..60_000 { state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407); scattered.insert(u128::from(state)); }
        assert_eq!(scattered.check_invariants(), 60_000);
        println!("Q2RESULT name=selftest policy={policy:?} clustered_levels={} scattered_levels={}", clustered.levels, scattered.levels);
    }
    let mut world = World::after_first_transaction(SplitPolicy::Half);
    let overwrite = world.publish(Operation::OverwriteFirstUnit);
    assert_eq!((overwrite.meta_blocks, overwrite.meta_slots), (7, 8), "里程碑二步 1：六个提交内生块 + 树表单元，inode 叶容器占两槽");
    let mut world = World::after_first_transaction(SplitPolicy::Half);
    let empty = world.publish(Operation::Empty);
    assert_eq!((empty.meta_blocks, empty.meta_slots), (4, 4), "D16（发布语义） 已定项 9：记账、分配记录、映射、树表各一");
    println!("Q2RESULT name=selftest first_overwrite={} first_empty={}", report_text(&overwrite), report_text(&empty));
    println!(
        "Q2RESULT name=capacities allocation_leaf={} allocation_internal={} mapping_leaf={} mapping_internal={} extent_leaf={} extent_internal={} accounting_leaf={}",
        leaf_capacity(10, 20), internal_capacity(10), leaf_capacity(27, 55), internal_capacity(27), leaf_capacity(24, 112), internal_capacity(24), leaf_capacity(22, 34)
    );
}

fn run_workload(name: &str, policy: SplitPolicy, operations: &[Operation]) {
    let mut world = World::after_first_transaction(policy);
    let mut previous_start = world.levels();
    let mut first_hits: BTreeMap<(usize, usize, bool), (u64, u64, String)> = BTreeMap::new();
    let mut deficit_publishes = [[0u64; 2]; 4];
    let mut worst_deficit_slots = [[0i64; 2]; 4];
    let mut first_path_excess: Option<String> = None;
    let mut first_fixpoint_split: Option<String> = None;
    let mut previous_start_of_empty_trace = Levels { extent: 0, allocation: 0, accounting: 0, mapping: 0 };
    for operation in operations {
        let report = world.publish(*operation);
        if *operation == Operation::Empty && report.levels_before != previous_start_of_empty_trace {
            previous_start_of_empty_trace = report.levels_before;
            let values: Vec<String> = TREE_SETS.iter().flat_map(|set| ARMS.iter().map(move |arm| (arm, set)))
                .map(|(arm, set)| format!("{arm:?}/{set:?}={}", arm_value(*arm, *set, previous_start, &report))).collect();
            println!("Q2RESULT name=empty_publish workload={name} {} arm_values={}", report_text(&report), values.join(","));
        }
        for (arm_position, arm) in ARMS.iter().enumerate() {
            for (set_position, set) in TREE_SETS.iter().enumerate() {
                let value = arm_value(*arm, *set, previous_start, &report);
                for by_slots in [false, true] {
                    let actual = if by_slots { report.meta_slots } else { report.meta_blocks };
                    if actual > value { first_hits.entry((arm_position, set_position, by_slots)).or_insert((value, actual, report_text(&report))); }
                }
                let deficit = i64::try_from(report.meta_slots).expect("槽") - i64::try_from(value).expect("块");
                if deficit > 0 { deficit_publishes[arm_position][set_position] += 1; }
                worst_deficit_slots[arm_position][set_position] = worst_deficit_slots[arm_position][set_position].max(deficit);
            }
        }
        let levels = report.levels_after;
        let path_only = [levels.extent, levels.allocation, levels.accounting, levels.mapping].iter().zip(report.trees_dirtied.iter())
            .map(|(tree_levels, dirtied)| if *dirtied { u64::try_from(*tree_levels).expect("层") } else { 0 }).sum::<u64>()
            + if report.inode_dirtied { 3 } else { 0 } + 1;
        if report.meta_slots > path_only && first_path_excess.is_none() {
            first_path_excess = Some(format!("path_only_slots={path_only} {}", report_text(&report)));
        }
        if report.splits_in_fixpoint > 0 && first_fixpoint_split.is_none() {
            let values: Vec<String> = ARMS.iter().map(|arm| format!("{:?}={}", arm, arm_value(*arm, TreeSet::JiaEveryRootedTreePlusMapping, previous_start, &report))).collect();
            first_fixpoint_split = Some(format!("{} jia_set_values={}", report_text(&report), values.join(",")));
        }
        previous_start = report.levels_before;
    }
    for ((arm_position, set_position, by_slots), (value, actual, text)) in &first_hits {
        println!("Q2RESULT name=first_hit workload={name} arm={:?} set={:?} unit={} arm_value={value} actual={actual} {text}",
            ARMS[*arm_position], TREE_SETS[*set_position], if *by_slots { "slots" } else { "blocks" });
    }
    for (arm_position, arm) in ARMS.iter().enumerate() {
        for (set_position, set) in TREE_SETS.iter().enumerate() {
            println!("Q2RESULT name=summary workload={name} arm={arm:?} set={set:?} publishes={} deficit_publishes_slots={} worst_deficit_slots={}",
                operations.len(), deficit_publishes[arm_position][set_position], worst_deficit_slots[arm_position][set_position]);
        }
    }
    println!("Q2RESULT name=first_path_excess workload={name} {}", first_path_excess.unwrap_or_else(|| "none".to_string()));
    println!("Q2RESULT name=first_fixpoint_split workload={name} {}", first_fixpoint_split.unwrap_or_else(|| "none".to_string()));
    println!("Q2RESULT name=final workload={name} levels={} allocation_records={} mapping_entries={}", levels_text(world.levels()), world.allocation.check_invariants(), world.mapping.check_invariants());
}

fn main() {
    self_tests();
    let repeat = |operation: Operation, count: usize| vec![operation; count];
    run_workload("overwrite_x3000", SplitPolicy::Half, &repeat(Operation::OverwriteFirstUnit, 3000));
    run_workload("append1_x3000", SplitPolicy::Half, &repeat(Operation::Append(1), 3000));
    run_workload("append1_x3000_right_end_split", SplitPolicy::RightEndAppend, &repeat(Operation::Append(1), 3000));
    run_workload("append64_x300", SplitPolicy::Half, &repeat(Operation::Append(64), 300));
    let mut grow_then_empty = repeat(Operation::Append(64), 100);
    grow_then_empty.extend(repeat(Operation::Empty, 20));
    run_workload("append64_x100_then_empty_x20", SplitPolicy::Half, &grow_then_empty);
    shortest_hit_on_bing_with_broad_tree_set();
    println!("Q2RESULT name=done");
}

/// 丙（上一次发布的高 + 每棵树 1 层）配甲的树集合是三臂里最宽的一格：对每个「每次发布追加 n 个单元」的固定脚本，
/// 找它第一次扣少（块数或槽数）的那次发布，报发布次数最少的 n 与那一格的数。
fn shortest_hit_on_bing_with_broad_tree_set() {
    let mut best: Option<(u64, u32, String, u64)> = None;
    for units in 1..=512u32 {
        let mut world = World::after_first_transaction(SplitPolicy::Half);
        for publish_index in 1..=60u64 {
            let report = world.publish(Operation::Append(units));
            let value = arm_value(Arm::BingPublishedPlusOneLevelEach, TreeSet::JiaEveryRootedTreePlusMapping, report.levels_before, &report);
            if report.meta_slots > value {
                if best.as_ref().map_or(true, |(best_index, best_units, _, _)| (publish_index, units) < (*best_index, *best_units)) {
                    best = Some((publish_index, units, report_text(&report), value));
                }
                break;
            }
        }
    }
    match best {
        Some((publishes, units, text, value)) => println!("Q2RESULT name=shortest_bing_broad_set_hit publishes={publishes} units_per_publish={units} arm_value={value} {text}"),
        None => println!("Q2RESULT name=shortest_bing_broad_set_hit none_within=60_publishes_x_512_units"),
    }
}
