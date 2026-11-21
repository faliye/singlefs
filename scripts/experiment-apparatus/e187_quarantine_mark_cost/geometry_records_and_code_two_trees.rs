//! 池几何与臂、被测核对器、单元与分配记录、码 2 的节点与插入删除、各级单元的收集。

use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct ShareTierBounds {
    pub(super) tier_two_from: u64,
    pub(super) tier_three_from: u64,
}

pub(super) fn usize_of(value: u64) -> usize {
    usize::try_from(value).expect("模型里的计数装得进 usize")
}

pub(super) fn u64_of(value: usize) -> u64 {
    u64::try_from(value).expect("模型里的计数装得进 u64")
}

/// 两个几何（登记第五节「池」）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum PoolGeometry {
    DefaultFourGibibyteDevices,
    WideOneTebibyteDevices,
}
pub(super) const GEOMETRIES: [PoolGeometry; 2] = [PoolGeometry::DefaultFourGibibyteDevices, PoolGeometry::WideOneTebibyteDevices];

impl PoolGeometry {
    pub(super) fn label(self) -> &'static str {
        match self {
            PoolGeometry::DefaultFourGibibyteDevices => "geometry_default",
            PoolGeometry::WideOneTebibyteDevices => "geometry_wide",
        }
    }

    pub(super) fn slots_per_device(self) -> u64 {
        match self {
            PoolGeometry::DefaultFourGibibyteDevices => (4u64 << 30) / SLOT_BYTES,
            PoolGeometry::WideOneTebibyteDevices => (1024u64 << 30) / SLOT_BYTES,
        }
    }
}

/// 层级 L 上一个分配记录树节点罩几个槽：W × F^L（`allocation_record_tree.rs` 的 `span_in_slots_at_level`）。
pub(super) fn span_in_slots_at_level(level: u8) -> u64 {
    (0..level).fold(ALLOCATION_RECORD_TREE_LEAF_SLOTS, |span, _| span.saturating_mul(ALLOCATION_RECORD_TREE_INTERNAL_FANOUT))
}

/// 根的层级：最小的 R ≥ 1 使 Σ_盘 ⌈盘上槽数 ÷ S_{R−1}⌉ ≤ F（`allocation_record_tree.rs` 文件头）。
pub(super) fn allocation_record_tree_root_level(slots_per_device: u64) -> u8 {
    (1u8..=u8::MAX)
        .find(|candidate| DEVICE_COUNT * slots_per_device.div_ceil(span_in_slots_at_level(candidate - 1)) <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT)
        .expect("6 字节槽号的盘在几层之内就装得进根")
}

/// 被比的三条臂与只给 `no_mark` 当阳性对照的参照臂（登记第五节「臂」）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum Arm {
    NoMark,
    SpanBit,
    QuarantineTree,
    PlainRelease,
}
pub(super) const FORK_ARMS: [Arm; 3] = [Arm::NoMark, Arm::SpanBit, Arm::QuarantineTree];
pub(super) const MARKING_ARMS: [Arm; 2] = [Arm::SpanBit, Arm::QuarantineTree];

impl Arm {
    pub(super) fn label(self) -> &'static str {
        match self {
            Arm::NoMark => "no_mark",
            Arm::SpanBit => "span_bit",
            Arm::QuarantineTree => "quarantine_tree",
            Arm::PlainRelease => "plain_release",
        }
    }
}

/// checker 怎么认隔离：`no_mark` 照今天的副本判据；两种丙各有两个变体（登记第五节「丙的 checker」）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum CheckerUnderTest {
    Today,
    MarkOnly,
    MarkOrToday,
}

impl CheckerUnderTest {
    pub(super) fn label(self) -> &'static str {
        match self {
            CheckerUnderTest::Today => "today",
            CheckerUnderTest::MarkOnly => "mark_only",
            CheckerUnderTest::MarkOrToday => "mark_or_today",
        }
    }
}

pub(super) fn checkers_of(arm: Arm) -> &'static [CheckerUnderTest] {
    match arm {
        Arm::NoMark | Arm::PlainRelease => &[CheckerUnderTest::Today],
        Arm::SpanBit | Arm::QuarantineTree => &[CheckerUnderTest::MarkOnly, CheckerUnderTest::MarkOrToday],
    }
}

/// 被隔离单元两份副本在盘上的样子（登记第一节 `quarantine_intact` / `quarantine_garbled`）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum CopyCondition {
    Intact,
    Garbled,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(super) struct UnitIdentifier(pub(super) u64);

/// 写量按结构分（登记第六节「分结构报」）。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum WrittenStructure {
    DataUnit,
    ExtentNode,
    InodeNode,
    AllocationRecordTreeNode,
    AccountingNode,
    QuarantineTreeNode,
    CentralMappingNode,
    TreeTableUnit,
}
const WRITTEN_STRUCTURES: [WrittenStructure; 8] = [
    WrittenStructure::DataUnit,
    WrittenStructure::ExtentNode,
    WrittenStructure::InodeNode,
    WrittenStructure::AllocationRecordTreeNode,
    WrittenStructure::AccountingNode,
    WrittenStructure::QuarantineTreeNode,
    WrittenStructure::CentralMappingNode,
    WrittenStructure::TreeTableUnit,
];

impl WrittenStructure {
    pub(super) fn label(self) -> &'static str {
        match self {
            WrittenStructure::DataUnit => "data",
            WrittenStructure::ExtentNode => "extent",
            WrittenStructure::InodeNode => "inode",
            WrittenStructure::AllocationRecordTreeNode => "allocation_record_tree",
            WrittenStructure::AccountingNode => "accounting",
            WrittenStructure::QuarantineTreeNode => "quarantine_tree",
            WrittenStructure::CentralMappingNode => "central_mapping",
            WrittenStructure::TreeTableUnit => "tree_table",
        }
    }

    pub(super) fn position(self) -> usize {
        WRITTEN_STRUCTURES.iter().position(|structure| *structure == self).expect("每个成员都在 WRITTEN_STRUCTURES 里")
    }
}

/// 中央映射 key 的排序五段：类、出生树、出生 txg、实例、尾段（`records.rs` 的 `mapping_key_sort_key`）。
pub(super) type MappingKey = (u8, u64, u64, u32, u64);
/// 乙的隔离树条目 key：(设备, 槽号)。
pub(super) type QuarantineEntryKey = (u32, u64);

/// 一个单元：住哪、几槽、进不进映射（进的带 key）、算进哪一类写。
#[derive(Clone, Copy, Debug)]
pub(super) struct UnitInformation {
    pub(super) slot: u64,
    pub(super) span_slots: u64,
    pub(super) mapping_key: Option<MappingKey>,
    pub(super) structure: WrittenStructure,
    pub(super) birth_txg: u64,
}

/// 一条分配记录（两块盘同形，只在位翻转分类表里分盘）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct AllocationRecord {
    pub(super) span_slots: u16,
    pub(super) is_released: bool,
    /// 仍分配时是分配代；已释放时是释放代。
    pub(super) generation: u64,
    pub(super) carries_the_quarantine_mark: bool,
}

impl AllocationRecord {
    pub(super) fn span_field(&self) -> u16 {
        let released_bit = if self.is_released { ALLOCATION_RECORD_RELEASED_FLAG } else { 0 };
        let mark_bit = if self.carries_the_quarantine_mark { QUARANTINE_MARK_BIT } else { 0 };
        self.span_slots | released_bit | mark_bit
    }
}

/// 分配记录树的一个节点：层级（0 是叶）、属于哪块盘、同盘同层按位置第几个；根记成层级 = 根层级、盘 0、第 0 个。
/// 派生的次序 (层级, 盘, 序号) 就是 bump 次序里「树内先叶后根、同层按 key 升序」那一段。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) struct AllocationNodePosition {
    pub(super) level: u8,
    pub(super) device: u32,
    pub(super) index_in_device: u64,
}

/// 一片叶（两块盘同一个位置）及它的全部祖先与根：叶一变，父指针一路变到根（`allocation_record_tree.rs` 的 `nodes_whose_contents_changed`）。
pub(super) fn leaf_and_its_ancestors(root_level: u8, leaf_index: u64, positions: &mut BTreeSet<AllocationNodePosition>) {
    for device in 0..u32::try_from(DEVICE_COUNT).expect("两块盘") {
        positions.insert(AllocationNodePosition {
            level: 0,
            device,
            index_in_device: leaf_index,
        });
        for level in 1..root_level {
            let index_in_device = leaf_index * ALLOCATION_RECORD_TREE_LEAF_SLOTS / span_in_slots_at_level(level);
            positions.insert(AllocationNodePosition {
                level,
                device,
                index_in_device,
            });
        }
    }
    positions.insert(AllocationNodePosition { level: root_level, device: 0, index_in_device: 0 });
}

pub(super) fn leaf_of_slot(slot: u64) -> u64 {
    slot / ALLOCATION_RECORD_TREE_LEAF_SLOTS
}

/// 码 2 树的容量（叶条目数、内部节点孩子数）。
#[derive(Clone, Copy, Debug)]
pub(super) struct CodeTwoCapacity {
    pub(super) leaf_entries: usize,
    pub(super) internal_entries: usize,
}

pub(super) fn mapping_capacity() -> CodeTwoCapacity {
    CodeTwoCapacity { leaf_entries: usize_of(MAPPING_LEAF_ENTRIES), internal_entries: usize_of(MAPPING_INTERNAL_ENTRIES) }
}

pub(super) fn quarantine_tree_capacity() -> CodeTwoCapacity {
    CodeTwoCapacity { leaf_entries: usize_of(QUARANTINE_TREE_LEAF_ENTRIES), internal_entries: usize_of(QUARANTINE_TREE_INTERNAL_ENTRIES) }
}

/// 这一次发布发出去的单元号：从 `first_fresh` 起都是这次新写的，比它小的都是上一版照抄来的。
#[derive(Clone, Copy, Debug)]
pub(super) struct FreshUnits {
    first_fresh: u64,
    pub(super) next_unit: u64,
}

impl FreshUnits {
    pub(super) fn starting_at(next_unit: u64) -> FreshUnits {
        FreshUnits { first_fresh: next_unit, next_unit }
    }

    pub(super) fn issue(&mut self) -> UnitIdentifier {
        let unit = UnitIdentifier(self.next_unit);
        self.next_unit += 1;
        unit
    }

    pub(super) fn is_fresh(&self, unit: UnitIdentifier) -> bool {
        unit.0 >= self.first_fresh
    }
}

/// 一棵码 2 树的一个节点（中央映射树与乙的隔离树共用这一份规则，`code_two_tree.rs` 文件头 ①–④）。
/// 节点体用 `Rc` 共享：一次发布在上一版的拷贝上改，只有改到的那几个节点真复制。
#[derive(Clone, Debug)]
pub(super) struct CodeTwoNode<Key: Ord + Clone> {
    pub(super) unit: UnitIdentifier,
    pub(super) body: Rc<CodeTwoBody<Key>>,
}

#[derive(Clone, Debug)]
pub(super) enum CodeTwoBody<Key: Ord + Clone> {
    Leaf(Vec<Key>),
    Internal { level: u8, children: Vec<CodeTwoChild<Key>> },
}

#[derive(Clone, Debug)]
pub(super) struct CodeTwoChild<Key: Ord + Clone> {
    separator_key: Key,
    pub(super) node: CodeTwoNode<Key>,
}

impl<Key: Ord + Clone> CodeTwoNode<Key> {
    fn level(&self) -> u8 {
        match &*self.body {
            CodeTwoBody::Leaf(_) => 0,
            CodeTwoBody::Internal { level, .. } => *level,
        }
    }

    pub(super) fn entry_count(&self) -> usize {
        match &*self.body {
            CodeTwoBody::Leaf(keys) => keys.len(),
            CodeTwoBody::Internal { children, .. } => children.len(),
        }
    }

    fn smallest_key(&self) -> Option<Key> {
        match &*self.body {
            CodeTwoBody::Leaf(keys) => keys.first().cloned(),
            CodeTwoBody::Internal { children, .. } => children.first().map(|child| child.separator_key.clone()),
        }
    }

    /// 这次发布改到它：照 COW 换一个新单元（同一次发布里只换一次）。
    fn mark_rewritten(&mut self, fresh: &mut FreshUnits) {
        if !fresh.is_fresh(self.unit) {
            self.unit = fresh.issue();
        }
    }
}

/// 插入用：最后一个分隔 key ≤ key 的孩子；key 比第一个分隔 key 还小时落到第一个孩子并把它的分隔 key 压低（已定项 11 ④）。
fn route_for_insertion<Key: Ord + Clone>(children: &mut [CodeTwoChild<Key>], key: &Key) -> usize {
    match children.iter().rposition(|child| child.separator_key <= *key) {
        Some(child_index) => child_index,
        None => {
            children[0].separator_key = key.clone();
            0
        }
    }
}

fn route_for_deletion<Key: Ord + Clone>(children: &[CodeTwoChild<Key>], key: &Key) -> usize {
    children.iter().rposition(|child| child.separator_key <= *key).unwrap_or(0)
}

/// 从中间切：左半留 ⌈n ÷ 2⌉ 条，右半是新节点，分隔 key 取右半的最小 key（已定项 11 ②④）。
fn split_in_the_middle<Key: Ord + Clone>(node: &mut CodeTwoNode<Key>, fresh: &mut FreshUnits) -> CodeTwoChild<Key> {
    let right_body = match Rc::make_mut(&mut node.body) {
        CodeTwoBody::Leaf(keys) => CodeTwoBody::Leaf(keys.split_off(keys.len().div_ceil(2))),
        CodeTwoBody::Internal { level, children } => {
            CodeTwoBody::Internal { level: *level, children: children.split_off(children.len().div_ceil(2)) }
        }
    };
    let right_half = CodeTwoNode { unit: fresh.issue(), body: Rc::new(right_body) };
    CodeTwoChild { separator_key: right_half.smallest_key().expect("切出来的右半不空"), node: right_half }
}

fn insert_below<Key: Ord + Clone>(
    node: &mut CodeTwoNode<Key>,
    key: Key,
    capacity: CodeTwoCapacity,
    fresh: &mut FreshUnits,
) -> Option<CodeTwoChild<Key>> {
    node.mark_rewritten(fresh);
    let overflows = match Rc::make_mut(&mut node.body) {
        CodeTwoBody::Leaf(keys) => {
            let insertion_index = keys.partition_point(|existing| *existing < key);
            assert!(keys.get(insertion_index) != Some(&key), "插入的 key 不在树里：插入集合 = 这一版的 key 减上一版的 key");
            keys.insert(insertion_index, key);
            keys.len() > capacity.leaf_entries
        }
        CodeTwoBody::Internal { children, .. } => {
            let child_index = route_for_insertion(children, &key);
            if let Some(right_half) = insert_below(&mut children[child_index].node, key, capacity, fresh) {
                children.insert(child_index + 1, right_half);
            }
            children.len() > capacity.internal_entries
        }
    };
    overflows.then(|| split_in_the_middle(node, fresh))
}

/// 在根上插一把 key：没有根时长一片新叶；根装不下时切开、长出新根（树高 + 1）。
pub(super) fn insert_at_the_root<Key: Ord + Clone>(
    root: &mut Option<CodeTwoNode<Key>>,
    key: Key,
    capacity: CodeTwoCapacity,
    fresh: &mut FreshUnits,
) {
    let Some(node) = root.as_mut() else {
        *root = Some(CodeTwoNode { unit: fresh.issue(), body: Rc::new(CodeTwoBody::Leaf(vec![key])) });
        return;
    };
    let Some(right_half) = insert_below(node, key, capacity, fresh) else {
        return;
    };
    let new_level = node.level().checked_add(1).expect("层级是 1 字节：模型里的树长不到 256 层");
    let left_half = node.clone();
    let left_separator = left_half.smallest_key().expect("刚切开的左半不空");
    *node = CodeTwoNode {
        unit: fresh.issue(),
        body: Rc::new(CodeTwoBody::Internal {
            level: new_level,
            children: vec![CodeTwoChild { separator_key: left_separator, node: left_half }, right_half],
        }),
    };
}

/// 删一把 key；删空的孩子从父节点里摘掉，不合并、不借（已定项 11 ③）。交回找没找到。
fn delete_below<Key: Ord + Clone>(node: &mut CodeTwoNode<Key>, key: &Key, fresh: &mut FreshUnits) -> bool {
    let found = match Rc::make_mut(&mut node.body) {
        CodeTwoBody::Leaf(keys) => match keys.binary_search(key) {
            Ok(index) => {
                keys.remove(index);
                true
            }
            Err(_) => false,
        },
        CodeTwoBody::Internal { children, .. } => {
            let child_index = route_for_deletion(children, key);
            let found_below = delete_below(&mut children[child_index].node, key, fresh);
            if found_below && children[child_index].node.entry_count() == 0 {
                children.remove(child_index);
            }
            found_below
        }
    };
    if found {
        node.mark_rewritten(fresh);
    }
    found
}

/// 根删完之后怎么收：根空了（零根，根指针写全零）、只剩一个孩子（孩子当根、孩子不重写）、照旧。
enum RootAfterADeletion<Key: Ord + Clone> {
    Stays,
    BecomesEmpty,
    CollapsesTo(CodeTwoNode<Key>),
}

pub(super) fn delete_at_the_root<Key: Ord + Clone>(root: &mut Option<CodeTwoNode<Key>>, key: &Key, fresh: &mut FreshUnits) {
    let node = root.as_mut().expect("删的 key 在树里：树不空");
    assert!(delete_below(node, key, fresh), "删的 key 在树里：删除集合 = 上一版的 key 减这一版的 key");
    // 迭代上界：每一轮树高减一，至多根的层级那么多轮。
    loop {
        let outcome = match root.as_ref().map(|root_node| &*root_node.body) {
            None => RootAfterADeletion::Stays,
            Some(CodeTwoBody::Leaf(keys)) if keys.is_empty() => RootAfterADeletion::BecomesEmpty,
            Some(CodeTwoBody::Leaf(_)) => RootAfterADeletion::Stays,
            Some(CodeTwoBody::Internal { children, .. }) if children.is_empty() => RootAfterADeletion::BecomesEmpty,
            Some(CodeTwoBody::Internal { children, .. }) if children.len() == 1 => {
                RootAfterADeletion::CollapsesTo(children[0].node.clone())
            }
            Some(CodeTwoBody::Internal { .. }) => RootAfterADeletion::Stays,
        };
        match outcome {
            RootAfterADeletion::Stays => return,
            RootAfterADeletion::BecomesEmpty => {
                *root = None;
                return;
            }
            RootAfterADeletion::CollapsesTo(only_child) => *root = Some(only_child),
        }
    }
}

/// 这次发布之后树长什么样：上一版有、这一版没有的 key 先删（升序），再插这一版新有的（升序）（`code_two_tree.rs` 的 `plan_the_tree_after_this_publish`）。
pub(super) fn plan_the_tree_after_this_publish<Key: Ord + Clone>(
    previous: &Option<CodeTwoNode<Key>>,
    deleted: &BTreeSet<Key>,
    inserted: &BTreeSet<Key>,
    capacity: CodeTwoCapacity,
    fresh: &mut FreshUnits,
) -> Option<CodeTwoNode<Key>> {
    let mut root = previous.clone();
    for key in deleted {
        delete_at_the_root(&mut root, key, fresh);
    }
    for key in inserted {
        insert_at_the_root(&mut root, key.clone(), capacity, fresh);
    }
    root
}

fn collect_units_by_level<Key: Ord + Clone>(node: &CodeTwoNode<Key>, units_by_level: &mut Vec<Vec<UnitIdentifier>>) {
    if let CodeTwoBody::Internal { children, .. } = &*node.body {
        for child in children {
            collect_units_by_level(&child.node, units_by_level);
        }
    }
    let level = usize::from(node.level());
    if units_by_level.len() <= level {
        units_by_level.resize_with(level + 1, Vec::new);
    }
    units_by_level[level].push(node.unit);
}

/// 树的全部节点按 bump 次序：先叶后根，同层按 key 升序（`code_two_tree.rs` 的 `flatten`）。
pub(super) fn units_of_the_tree_in_bump_order<Key: Ord + Clone>(root: &Option<CodeTwoNode<Key>>) -> Vec<UnitIdentifier> {
    let mut units_by_level = Vec::new();
    if let Some(node) = root {
        collect_units_by_level(node, &mut units_by_level);
    }
    units_by_level.into_iter().flatten().collect()
}

/// 树里有没有这把 key（乙的 checker 读隔离树判记号齐不齐）。
pub(super) fn tree_contains<Key: Ord + Clone>(root: &Option<CodeTwoNode<Key>>, key: &Key) -> bool {
    let Some(mut node) = root.as_ref() else {
        return false;
    };
    loop {
        match &*node.body {
            CodeTwoBody::Leaf(keys) => return keys.binary_search(key).is_ok(),
            CodeTwoBody::Internal { children, .. } => node = &children[route_for_deletion(children, key)].node,
        }
    }
}

pub(super) fn tree_height<Key: Ord + Clone>(root: &Option<CodeTwoNode<Key>>) -> u8 {
    root.as_ref().map_or(0, |node| node.level() + 1)
}
