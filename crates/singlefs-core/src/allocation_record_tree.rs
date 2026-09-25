//! 分配记录树按 key 空间定形状（D8（核心索引结构） 已定项 14「分配记录树」，用户 2026-09-24 定 K1）：不分裂，按绝对槽号按位置寻址。
//!
//! - **叶**：盘 d 上第 k 片叶罩槽 `[k × W, (k + 1) × W)`，W = [`ALLOCATION_RECORD_TREE_LEAF_SLOTS`]（偶数、≤ 叶条目容量 812）；
//!   记录按起点槽落叶，记录的末槽不越过它所在叶的末槽（两槽单元起在偶数槽、W 取偶数 ⇒ 永远不跨叶）。
//! - **往上各层按位置寻址**：层级 L（L ≥ 1）上盘 d 第 j 个节点罩槽 `[j × S_L, (j + 1) × S_L)`，S_L = W × F^L，
//!   F = [`ALLOCATION_RECORD_TREE_INTERNAL_FANOUT`]（内部条目 96 = 孩子罩的那一段的起点 key 10 + 子指针 86）。子节点罩哪一段由它在父节点里的
//!   那一格算出：父节点的条目 key 就是孩子那一段的起点，checker 与读者都按位置独立算出孩子该罩的那一段逐节点核。
//! - **根**罩整个 key 空间（`[全 0, 全 FF]`，盘这一维也在根里分路）：根的层级 R 是池几何的确定函数——最小的 R ≥ 1 使
//!   Σ_盘 ⌈盘上绝对槽数 ÷ S_{R−1}⌉ ≤ F（根装得下每块盘上 R − 1 层那一层的全部格）。树高 = 根节点头里的层级 + 1（D8 已定项 11 ⑤）。
//! - **没有记录的一段 = 全空闲**：那片叶不写，父节点那一格留空；孩子一个都没有的内部节点同样不写。根恒在（池里恒有 mkfs 那两个单元的记录）。
//!
//! 盘这一维怎么进位置寻址（根里按盘分路、根之下每个节点只属于一块盘），条款没写到，是实现员的取法（交回里写明）。
//!
//! 这里只管结构：几何、这一版有哪些节点、哪些节点的内容这次变了、这次之后树长什么样、装一个节点的字节、从盘上读回一棵树并逐节点核位置。
//! 取落点、发出生序号、写盘在 `crate::transaction` 那一侧；交回拒绝的一样都不动盘。

use std::collections::{BTreeMap, BTreeSet};

use singlefs_format::{
    ALLOCATION_RECORD_BYTES, ALLOCATION_RECORD_KEY_BYTES,
    ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES, ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
    ALLOCATION_RECORD_TREE_LEAF_SLOTS,
};

use crate::address::{
    CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber, TreeIdentifier,
};
use crate::allocator::{absolute_slot_count_of_device, AllocationRecord, PoolAllocator};
use crate::bytes::{ByteReader, ByteWriter};
use crate::pointer::{BirthSequence, NodePointer};
use crate::recovery::{PoolReader, RecoveryFailure};
use crate::root_record::RootRecord;
use crate::unit::{build_index_node, parse_index_node, IndexNodeHeader};

/// 槽号字段是 6 字节（D3（空间分配） 已定项 7 的 key 段宽）：一块盘上最大的槽号。
const LARGEST_SLOT_NUMBER: u64 = (1 << 48) - 1;

/// 根之下一个节点的位置：层级（0 是叶）、它属于哪块盘、同一块盘同一层从槽号 0 起数第几个（按位置，不按有没有节点）。
/// 派生的全序（层级升序、同层按盘再按序号升序 = 按 key 升序）就是这棵树在一次发布里的 bump 次序与出生序号的发号次序：
/// 树内先叶后根、同层按 key 升序（D3（空间分配） 已定项 10 ⑤、D19（块指针的结构与宽度预算） 已定项 9）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AllocationRecordTreeNodePosition {
    pub level: u8,
    pub device: DeviceIdentity,
    pub index_in_device: u64,
}

/// 分配记录树里的一个节点：根之下的按位置，根只有一个。派生的全序把根排在最末（先叶后根）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AllocationRecordTreeNode {
    BelowTheRoot(AllocationRecordTreeNodePosition),
    Root,
}

/// 分配记录树的 key（设备 4, 槽号 6，盘上小端）的字节。
#[must_use]
pub fn allocation_record_key_bytes(device: DeviceIdentity, slot: SlotNumber) -> Vec<u8> {
    let mut writer = ByteWriter::new(usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10"));
    writer.put_u32(device.0);
    writer.put_six_byte_unsigned(slot.0);
    writer.into_bytes()
}

fn key_bytes_width() -> usize {
    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10")
}

fn parse_allocation_record_key(key: &[u8]) -> (DeviceIdentity, SlotNumber) {
    let mut reader = ByteReader::at(key, 0);
    (
        DeviceIdentity(reader.get_u32()),
        SlotNumber(reader.get_six_byte_unsigned()),
    )
}

/// 这个池上分配记录树的几何：池里每块盘的绝对槽数（按设备身份升序）与由它们定下的根层级。
/// 写者按分配器建空闲图时记下的每块盘字节数算（[`AllocationRecordTreeGeometry::of_allocator`]），读者按读者报的字节数算
/// （[`AllocationRecordTreeGeometry::of_reader`]），两侧同走 [`AllocationRecordTreeGeometry::of_device_sizes_in_bytes`]、
/// 盘上绝对槽数只有 [`absolute_slot_count_of_device`] 一处定义；checker 另写一份按同一条公式算。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllocationRecordTreeGeometry {
    slots_of_each_device: Vec<(DeviceIdentity, u64)>,
    root_level: u8,
}

impl AllocationRecordTreeGeometry {
    /// 从每块盘的绝对槽数（设备字节数 ÷ 16384）算。
    ///
    /// # Panics
    /// 池里一块盘都没有，或盘多到连 255 层的根都装不下每块盘各一格（多于 169 块盘）：池里恒有两块盘（mkfs 断言过）。
    #[must_use]
    pub fn of_devices(slots_of_each_device: &[(DeviceIdentity, u64)]) -> Self {
        assert!(
            !slots_of_each_device.is_empty(),
            "池里至少一块盘：mkfs 断言过两块"
        );
        let mut slots_of_each_device = slots_of_each_device.to_vec();
        slots_of_each_device.sort_by_key(|(device, _)| *device);
        // 迭代上界是 255 层；每一轮 S_{R−1} 乘 F，槽数有上界 2^48，至多 6 轮就到每块盘一格。
        let root_level = (1..=u8::MAX)
            .find(|candidate_root_level| {
                let child_span = span_in_slots_at_level(candidate_root_level - 1);
                let cells: u64 = slots_of_each_device
                    .iter()
                    .map(|(_, slots)| slots.div_ceil(child_span))
                    .sum();
                cells <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
            })
            .expect("盘数不多于扇出 169 时，槽数有上界 2^48、层级至多 6 就装得下每块盘各一格");
        Self {
            slots_of_each_device,
            root_level,
        }
    }

    /// 从每块盘的字节数算：盘上绝对槽数只有 [`absolute_slot_count_of_device`] 一处定义，写侧（[`Self::of_allocator`]）与
    /// 读侧（[`Self::of_reader`]）都走这里，两侧只差字节数从哪来（分配器建空闲图时记下的、读者报的）。
    #[must_use]
    pub fn of_device_sizes_in_bytes(device_sizes_in_bytes: &[(DeviceIdentity, u64)]) -> Self {
        let slots: Vec<(DeviceIdentity, u64)> = device_sizes_in_bytes
            .iter()
            .map(|(device, device_bytes)| (*device, absolute_slot_count_of_device(*device_bytes)))
            .collect();
        Self::of_devices(&slots)
    }

    /// 写者那一侧：分配器里每块盘建空闲图时的字节数（`DeviceFreeMap::device_size_in_bytes`）。
    #[must_use]
    pub fn of_allocator(allocator: &PoolAllocator) -> Self {
        let device_sizes_in_bytes: Vec<(DeviceIdentity, u64)> = allocator
            .devices
            .iter()
            .map(|device_map| (device_map.device, device_map.device_size_in_bytes()))
            .collect();
        Self::of_device_sizes_in_bytes(&device_sizes_in_bytes)
    }

    /// 读者那一侧：池里每块盘的字节数。
    ///
    /// # Panics
    /// 读者列出的盘说不出自己的字节数：两个实现都从同一张表里取（`PoolReader::device_size_in_bytes`）。
    #[must_use]
    pub fn of_reader<Reader: PoolReader + ?Sized>(reader: &Reader) -> Self {
        let device_sizes_in_bytes: Vec<(DeviceIdentity, u64)> = reader
            .device_identities()
            .into_iter()
            .map(|device| {
                (
                    device,
                    reader
                        .device_size_in_bytes(device)
                        .expect("读者列出的盘说得出自己的字节数"),
                )
            })
            .collect();
        Self::of_device_sizes_in_bytes(&device_sizes_in_bytes)
    }

    /// 根的层级（树高 − 1）。
    #[must_use]
    pub fn root_level(&self) -> u8 {
        self.root_level
    }

    /// 树高：根的层级 + 1（D8（核心索引结构） 已定项 11 ⑤）。
    #[must_use]
    pub fn height(&self) -> u64 {
        u64::from(self.root_level) + 1
    }

    /// 这块盘在不在池里。
    #[must_use]
    pub fn has_device(&self, device: DeviceIdentity) -> bool {
        self.slots_of_each_device
            .iter()
            .any(|(candidate, _)| *candidate == device)
    }

    /// 一个槽所在的那片叶。
    #[must_use]
    pub fn leaf_of_slot(
        &self,
        device: DeviceIdentity,
        slot: SlotNumber,
    ) -> AllocationRecordTreeNode {
        AllocationRecordTreeNode::BelowTheRoot(AllocationRecordTreeNodePosition {
            level: 0,
            device,
            index_in_device: slot.0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS,
        })
    }

    /// 一个节点的父节点；根没有父节点。
    #[must_use]
    pub fn parent_of(&self, node: AllocationRecordTreeNode) -> Option<AllocationRecordTreeNode> {
        match node {
            AllocationRecordTreeNode::Root => None,
            AllocationRecordTreeNode::BelowTheRoot(position) => {
                let parent_level = position.level + 1;
                if parent_level == self.root_level {
                    Some(AllocationRecordTreeNode::Root)
                } else {
                    Some(AllocationRecordTreeNode::BelowTheRoot(
                        AllocationRecordTreeNodePosition {
                            level: parent_level,
                            device: position.device,
                            index_in_device: position.index_in_device
                                / ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
                        },
                    ))
                }
            }
        }
    }

    /// 一个节点的层级。
    #[must_use]
    pub fn level_of(&self, node: AllocationRecordTreeNode) -> u8 {
        match node {
            AllocationRecordTreeNode::Root => self.root_level,
            AllocationRecordTreeNode::BelowTheRoot(position) => position.level,
        }
    }

    /// 一个节点按位置规定罩的那一段 key，(最小, 最大) 两把 key 的字节：节点头里的 key 区间就写它
    /// （D18（块里携带什么信息） 已定项 2：按位置寻址的树，节点头的 key 区间写这个节点按位置规定罩的那一段）。
    /// 根罩整个 key 空间；根之下的节点罩它那块盘上 `[j × S_L, (j + 1) × S_L)`（末端不越过 6 字节槽号的最大值）。
    #[must_use]
    pub fn key_range_of(&self, node: AllocationRecordTreeNode) -> (Vec<u8>, Vec<u8>) {
        match node {
            AllocationRecordTreeNode::Root => (
                vec![0u8; key_bytes_width()],
                vec![u8::MAX; key_bytes_width()],
            ),
            AllocationRecordTreeNode::BelowTheRoot(position) => {
                let (first_slot, last_slot) = slot_range_of(position);
                (
                    allocation_record_key_bytes(position.device, first_slot),
                    allocation_record_key_bytes(position.device, last_slot),
                )
            }
        }
    }

    /// 父节点里一条内部条目的 key 指的是哪个孩子：key 是孩子那一段的起点，槽号要落在孩子那一层的格点上；父节点在根之下时
    /// 孩子与父节点同盘、落在父节点那一段里。对不上交回 `None`（盘上读来的 key 可以是任何值）。
    #[must_use]
    pub fn child_named_by_entry_key(
        &self,
        parent: AllocationRecordTreeNode,
        entry_key: &[u8],
    ) -> Option<AllocationRecordTreeNode> {
        let parent_level = self.level_of(parent);
        let child_level = parent_level.checked_sub(1)?;
        let (device, slot) = parse_allocation_record_key(entry_key);
        let child_span = span_in_slots_at_level(child_level);
        if !slot.0.is_multiple_of(child_span) || !self.has_device(device) {
            return None;
        }
        let child = AllocationRecordTreeNodePosition {
            level: child_level,
            device,
            index_in_device: slot.0 / child_span,
        };
        let child_node = AllocationRecordTreeNode::BelowTheRoot(child);
        (self.parent_of(child_node) == Some(parent)).then_some(child_node)
    }

    /// 一条记录落在它所在的叶里：同盘、起点槽与末槽都在叶那一段里（D8 已定项 14「记录的末槽不越过它所在叶的末槽」）。
    #[must_use]
    pub fn record_fits_in_its_leaf(&self, record: &AllocationRecord) -> bool {
        let AllocationRecordTreeNode::BelowTheRoot(leaf) =
            self.leaf_of_slot(record.device, record.slot)
        else {
            return false;
        };
        let (_, last_slot) = slot_range_of(leaf);
        record.span_slots > 0 && record.slot.0 + u64::from(record.span_slots) - 1 <= last_slot.0
    }
}

/// 层级 L 上一个节点罩几个槽：W × F^L，乘到装不进 u64 就停在 u64 的最大值（那么宽的一层罩得住整个 6 字节槽号空间）。
#[must_use]
pub fn span_in_slots_at_level(level: u8) -> u64 {
    (0..level).fold(ALLOCATION_RECORD_TREE_LEAF_SLOTS, |span, _| {
        span.saturating_mul(ALLOCATION_RECORD_TREE_INTERNAL_FANOUT)
    })
}

/// 根之下一个节点罩的槽：`[j × S_L, (j + 1) × S_L)` 的首末两槽，末槽不越过 6 字节槽号的最大值。
fn slot_range_of(position: AllocationRecordTreeNodePosition) -> (SlotNumber, SlotNumber) {
    let span = span_in_slots_at_level(position.level);
    let first_slot = position.index_in_device.saturating_mul(span);
    let last_slot = first_slot.saturating_add(span - 1).min(LARGEST_SLOT_NUMBER);
    (
        SlotNumber(first_slot.min(LARGEST_SLOT_NUMBER)),
        SlotNumber(last_slot),
    )
}

/// 每片有记录的叶里装的记录，按 key 升序。
///
/// # Panics
/// 有一条记录越过它所在叶的末槽：写者发的落点两槽单元起在偶数槽、W 取偶数（[`ALLOCATION_RECORD_TREE_LEAF_SLOTS`] 的断言），
/// 从盘上读来的记录进分配器之前由 [`read_allocation_record_tree`] 逐条判过。
#[must_use]
pub fn records_of_each_leaf(
    geometry: &AllocationRecordTreeGeometry,
    records: &[AllocationRecord],
) -> BTreeMap<AllocationRecordTreeNode, Vec<AllocationRecord>> {
    let mut by_leaf: BTreeMap<AllocationRecordTreeNode, Vec<AllocationRecord>> = BTreeMap::new();
    for record in records {
        assert!(
            geometry.record_fits_in_its_leaf(record),
            "分配记录（盘 {:?} 槽 {:?} 跨 {}）越过它所在叶的末槽：两槽单元起在偶数槽、叶宽取偶数，盘上读来的记录进来之前判过",
            record.device,
            record.slot,
            record.span_slots
        );
        by_leaf
            .entry(geometry.leaf_of_slot(record.device, record.slot))
            .or_default()
            .push(*record);
    }
    for leaf_records in by_leaf.values_mut() {
        leaf_records.sort_by_key(AllocationRecord::sort_key);
    }
    by_leaf
}

/// 这些记录在树里要哪些节点：有记录的叶、它们的每一个祖先、根。
#[must_use]
pub fn nodes_holding_records(
    geometry: &AllocationRecordTreeGeometry,
    records: &[AllocationRecord],
) -> BTreeSet<AllocationRecordTreeNode> {
    let mut nodes = BTreeSet::new();
    nodes.insert(AllocationRecordTreeNode::Root);
    for leaf in records_of_each_leaf(geometry, records).into_keys() {
        insert_the_node_and_its_ancestors(geometry, leaf, &mut nodes);
    }
    nodes
}

fn insert_the_node_and_its_ancestors(
    geometry: &AllocationRecordTreeGeometry,
    node: AllocationRecordTreeNode,
    nodes: &mut BTreeSet<AllocationRecordTreeNode>,
) {
    let mut current = Some(node);
    // 迭代上界是根的层级 + 1：每一轮往上一层，根没有父节点。
    while let Some(ancestor) = current {
        if !nodes.insert(ancestor) && ancestor != node {
            break;
        }
        current = geometry.parent_of(ancestor);
    }
}

/// 从上一版的记录到这一版的记录，内容变了的节点：记录清单（逐字节）不同的叶——这一版新有、上一版有而这一版没有的都算——
/// 与它们的每一个祖先、根。叶一变，它这一版的指针就变，父节点那一格跟着变，一路到根（COW 叶 + 全部祖先）。
#[must_use]
pub fn nodes_whose_contents_changed(
    geometry: &AllocationRecordTreeGeometry,
    previous_records: &[AllocationRecord],
    records_after: &[AllocationRecord],
) -> BTreeSet<AllocationRecordTreeNode> {
    let before = records_of_each_leaf(geometry, previous_records);
    let after = records_of_each_leaf(geometry, records_after);
    let mut changed = BTreeSet::new();
    let leaves: BTreeSet<AllocationRecordTreeNode> =
        before.keys().chain(after.keys()).copied().collect();
    for leaf in leaves {
        if before.get(&leaf) != after.get(&leaf) {
            insert_the_node_and_its_ancestors(geometry, leaf, &mut changed);
        }
    }
    if !changed.is_empty() {
        changed.insert(AllocationRecordTreeNode::Root);
    }
    changed
}

/// 一版分配记录树里一个节点从哪来：照抄上一版同一个位置上的节点（字节、落点、指针一个不动），或这次重写。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllocationRecordTreeNodeOrigin {
    CarriedFromThePreviousVersion,
    RewrittenThisPublish,
}

/// 这次发布之后分配记录树长什么样。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllocationRecordTreePlan {
    /// 这一版的每个节点与它从哪来，按 bump 次序（根之下的按位置升序，根在最末）。
    pub nodes: Vec<(AllocationRecordTreeNode, AllocationRecordTreeNodeOrigin)>,
    /// 上一版里这次被换下的节点（重写了的、这一版不再有的），按上一版的 bump 次序。
    pub replaced_previous_nodes: Vec<AllocationRecordTreeNode>,
}

impl AllocationRecordTreePlan {
    /// 这次重写的节点，按 bump 次序（= 这棵树里出生序号的发号次序）。
    #[must_use]
    pub fn rewritten_nodes(&self) -> Vec<AllocationRecordTreeNode> {
        self.nodes
            .iter()
            .filter(|(_, origin)| *origin == AllocationRecordTreeNodeOrigin::RewrittenThisPublish)
            .map(|(node, _)| *node)
            .collect()
    }

    /// 这一版的全部节点，按 bump 次序。
    #[must_use]
    pub fn node_set(&self) -> BTreeSet<AllocationRecordTreeNode> {
        self.nodes.iter().map(|(node, _)| *node).collect()
    }
}

/// 这次之后树长什么样：这一版的节点是 `nodes_after`（有记录的叶、它们的祖先、根），其中在 `rewritten` 里的、或上一版没有的，这次重写；
/// 别的照抄上一版同一个位置上的节点。上一版有而这一版没有的、这次重写的，都换下。
///
/// `rewritten` 由调用方给：它要罩住内容变了的节点（[`nodes_whose_contents_changed`]），可以多罩（多罩的那几个内容没变、照样重写一份）。
/// 调用方在分配器的一份拷贝上把这次的释放与取落点走一遍、按走出来的记录算内容变了的节点，不在 `rewritten` 里就并进去再走一遍，直到罩住
/// （固定点：分配记录树的节点也要给自己记分配记录，取落点又会改记录）。
#[must_use]
pub fn plan_the_tree_after_this_publish(
    previous_nodes: &BTreeSet<AllocationRecordTreeNode>,
    nodes_after: &BTreeSet<AllocationRecordTreeNode>,
    rewritten: &BTreeSet<AllocationRecordTreeNode>,
) -> AllocationRecordTreePlan {
    let nodes = nodes_after
        .iter()
        .map(|node| {
            let origin = if rewritten.contains(node) || !previous_nodes.contains(node) {
                AllocationRecordTreeNodeOrigin::RewrittenThisPublish
            } else {
                AllocationRecordTreeNodeOrigin::CarriedFromThePreviousVersion
            };
            (*node, origin)
        })
        .collect();
    let replaced_previous_nodes = previous_nodes
        .iter()
        .filter(|node| rewritten.contains(node) || !nodes_after.contains(node))
        .copied()
        .collect();
    AllocationRecordTreePlan {
        nodes,
        replaced_previous_nodes,
    }
}

/// 一个内部节点这一版的孩子，按位置升序（= 按 key 升序）：`nodes` 是这一版的全部节点。
#[must_use]
pub fn children_of(
    geometry: &AllocationRecordTreeGeometry,
    nodes: &BTreeSet<AllocationRecordTreeNode>,
    parent: AllocationRecordTreeNode,
) -> Vec<AllocationRecordTreeNode> {
    nodes
        .iter()
        .copied()
        .filter(|node| geometry.parent_of(*node) == Some(parent))
        .collect()
}

/// 一版分配记录树在内存里的样子：每个节点与它这一版的指针，按 bump 次序（根之下的按位置升序，根在最末）。
/// 节点的字节住 `transaction::TransactionOutput::units` 里它那个角色的单元。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AllocationRecordTreeVersion {
    pub nodes: Vec<(AllocationRecordTreeNode, NodePointer)>,
}

impl AllocationRecordTreeVersion {
    /// 这一版的全部节点。
    #[must_use]
    pub fn node_set(&self) -> BTreeSet<AllocationRecordTreeNode> {
        self.nodes.iter().map(|(node, _)| *node).collect()
    }

    /// 这个节点这一版的指针；这一版没有它交回 `None`。
    #[must_use]
    pub fn pointer_of(&self, node: AllocationRecordTreeNode) -> Option<NodePointer> {
        self.nodes
            .iter()
            .find(|(candidate, _)| *candidate == node)
            .map(|(_, pointer)| *pointer)
    }

    /// 根的指针（树表条目或根记录那一项里写的那一条）。
    ///
    /// # Panics
    /// 这一版没有根：有记录的一版恒有根（池里恒有 mkfs 那两个单元的记录）。
    #[must_use]
    pub fn root_pointer(&self) -> NodePointer {
        self.pointer_of(AllocationRecordTreeNode::Root)
            .expect("有记录的一版恒有根")
    }
}

/// 一个节点装的东西：叶装记录（按 key 升序），内部节点装孩子与孩子这一版的指针（按位置升序）。
pub enum AllocationRecordTreeNodeContents<'contents> {
    Leaf(&'contents [AllocationRecord]),
    Internal(Vec<(AllocationRecordTreeNode, NodePointer)>),
}

/// 装一个分配记录树节点的字节（码 2，D18（块里携带什么信息） 已定项 18 的头）：key 区间写这个节点按位置规定罩的那一段；
/// 叶的条目是分配记录 20，内部节点的条目是「孩子那一段的起点 key + 孩子这一版的指针」96。
///
/// # Panics
/// 叶里一条记录都没有（缺席的叶不写），或内部节点一个孩子都没有：规划只交出有记录的叶与它们的祖先。
#[allow(
    clippy::too_many_arguments,
    reason = "装一个节点要的：几何、哪个节点、装什么、树号，与码 2 头的四样身份字段，各自独立"
)]
#[must_use]
pub fn build_allocation_record_tree_node(
    geometry: &AllocationRecordTreeGeometry,
    node: AllocationRecordTreeNode,
    contents: &AllocationRecordTreeNodeContents<'_>,
    tree: TreeIdentifier,
    birth_txg: CheckpointTxg,
    filesystem_identifier: &[u8; 16],
    instance: InstanceGeneration,
    birth_sequence: BirthSequence,
) -> Vec<u8> {
    let (smallest_key, largest_key) = geometry.key_range_of(node);
    let (entry_width, entries): (u64, Vec<Vec<u8>>) = match contents {
        AllocationRecordTreeNodeContents::Leaf(records) => {
            assert!(!records.is_empty(), "缺席的叶不写：规划只交出有记录的叶");
            (
                ALLOCATION_RECORD_BYTES,
                records.iter().map(AllocationRecord::to_bytes).collect(),
            )
        }
        AllocationRecordTreeNodeContents::Internal(children) => {
            assert!(
                !children.is_empty(),
                "孩子一个都没有的内部节点不写：规划只交出有记录的叶的祖先"
            );
            (
                ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES,
                children
                    .iter()
                    .map(|(child, pointer)| {
                        let (child_start, _) = geometry.key_range_of(*child);
                        let mut writer = ByteWriter::new(
                            usize::try_from(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES)
                                .expect("96"),
                        );
                        writer.put(&child_start);
                        pointer.write_to(&mut writer);
                        writer.into_bytes()
                    })
                    .collect(),
            )
        }
    };
    build_index_node(
        tree,
        geometry.level_of(node),
        key_bytes_width(),
        &smallest_key,
        &largest_key,
        birth_txg,
        filesystem_identifier,
        instance,
        birth_sequence,
        u16::try_from(entry_width).expect("条目宽 2 字节"),
        &entries,
    )
}

/// 从盘上读一棵分配记录树时核到哪一步（同 `code_two_tree::CodeTwoTreeHeaderJudgement` 的两档）。**位置那几样两档都核**：
/// 节点的层级、节点头里的 key 区间是不是它的位置规定的那一段、父节点的条目 key 是不是一个合法孩子的起点、记录落不落在它所在叶里——
/// 这一版在内存里按位置记节点、按叶分记录，读回来的位置对不上，下一次发布算「哪片叶变了」就算错。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllocationRecordTreeHeaderJudgement {
    /// 另核每个节点头的树 ID、fsid、诞生不晚于根、出生序号与指着它的指针相同（冷走读、影子账）。
    EveryHeaderAgainstItsReference,
    /// 头里的出生身份不核（从盘上重建上一版、挂载时重建分配器，与重建路径读别的树同一个口径）。
    OnlyWhatThePositionsNeed,
}

/// 从盘上读回来的一棵分配记录树：每个节点、它的指针与字节（bump 次序），与全部记录（按 key 升序）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllocationRecordTreeReadFromDisk {
    pub nodes: Vec<(AllocationRecordTreeNode, NodePointer, Vec<u8>)>,
    pub records: Vec<AllocationRecord>,
}

impl AllocationRecordTreeReadFromDisk {
    /// 读回来的这一版（节点与指针）。
    #[must_use]
    pub fn version(&self) -> AllocationRecordTreeVersion {
        AllocationRecordTreeVersion {
            nodes: self
                .nodes
                .iter()
                .map(|(node, pointer, _)| (*node, *pointer))
                .collect(),
        }
    }
}

fn violated(detail: &'static str) -> RecoveryFailure {
    RecoveryFailure::InvariantViolated {
        invariant: "I-1.1",
        detail,
    }
}

/// 读一棵分配记录树时一路不变的那几样。
struct AllocationRecordTreeReading<'reading> {
    geometry: &'reading AllocationRecordTreeGeometry,
    tree: TreeIdentifier,
    judgement: AllocationRecordTreeHeaderJudgement,
    root: &'reading RootRecord,
    expected_filesystem_identifier: u64,
}

impl AllocationRecordTreeReading<'_> {
    /// 核一个节点头里的出生身份（只有 `EveryHeaderAgainstItsReference` 才核）。根那一个的报错逐字沿用 `recovery` 核树根的那几句
    /// （同 `code_two_tree` 的做法），读路径上几种坏法报的成员不随树长没长多层而变。
    fn judge_the_identity_in_the_header(
        &self,
        header: &IndexNodeHeader,
        pointer: &NodePointer,
        is_root: bool,
    ) -> Result<(), RecoveryFailure> {
        match self.judgement {
            AllocationRecordTreeHeaderJudgement::OnlyWhatThePositionsNeed => return Ok(()),
            AllocationRecordTreeHeaderJudgement::EveryHeaderAgainstItsReference => {}
        }
        let detail = |of_the_root: &'static str, of_a_node_below_the_root: &'static str| {
            if is_root {
                of_the_root
            } else {
                of_a_node_below_the_root
            }
        };
        if header.tree != self.tree {
            return Err(RecoveryFailure::InvariantViolated {
                invariant: "I-1.3",
                detail: detail(
                    "根头里的树 ID 与树表不符",
                    "分配记录树节点头里的树 ID 与引用它的树不符",
                ),
            });
        }
        if header.key_width != key_bytes_width() {
            return Err(RecoveryFailure::InvariantViolated {
                invariant: "E142 走读同款",
                detail: detail(
                    "根自述 key 宽与树的种类不符",
                    "分配记录树节点自述 key 宽不是 10",
                ),
            });
        }
        if header.birth_txg > self.root.checkpoint_txg || header.instance > self.root.instance {
            return Err(RecoveryFailure::InvariantViolated {
                invariant: "I-1.2",
                detail: detail("树根诞生于根之后", "分配记录树节点诞生于根之后"),
            });
        }
        if header.filesystem_identifier != self.expected_filesystem_identifier {
            return Err(RecoveryFailure::InvariantViolated {
                invariant: "I-1.4",
                detail: detail("树根 fsid 不符", "分配记录树节点 fsid 不符"),
            });
        }
        if header.birth_sequence != pointer.birth_sequence {
            return Err(RecoveryFailure::InvariantViolated {
                invariant: "I-1.2",
                detail: detail("树根出生序号与指针不符", "分配记录树节点出生序号与指针不符"),
            });
        }
        Ok(())
    }

    /// 读一个节点和它下面整棵子树，按后序把 (节点, 指针, 字节) 推进 `nodes`、叶里的记录推进 `records`。
    fn read_node_and_its_subtree(
        &self,
        node: AllocationRecordTreeNode,
        pointer: &NodePointer,
        read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
        units_seen: &mut BTreeSet<SlotNumber>,
        nodes: &mut Vec<(AllocationRecordTreeNode, NodePointer, Vec<u8>)>,
        records: &mut Vec<AllocationRecord>,
    ) -> Result<(), RecoveryFailure> {
        if !units_seen.insert(pointer.locations[0].slot) {
            return Err(violated("同一个分配记录树节点被两条父条目引用"));
        }
        let bytes = read_node(pointer)?;
        let is_root = node == AllocationRecordTreeNode::Root;
        let header = parse_index_node(&bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: if is_root {
                "分配记录树根"
            } else {
                "分配记录树节点"
            },
        })?;
        self.judge_the_identity_in_the_header(&header, pointer, is_root)?;
        if header.key_width != key_bytes_width() {
            return Err(RecoveryFailure::InvariantViolated {
                invariant: "E142 走读同款",
                detail: "分配记录树节点自述 key 宽不是 10",
            });
        }
        if header.level != self.geometry.level_of(node) {
            return Err(violated("分配记录树节点的层级不是它的位置规定的那一层"));
        }
        let (smallest_key, largest_key) = self.geometry.key_range_of(node);
        if header.smallest_key != smallest_key || header.largest_key != largest_key {
            return Err(violated(
                "分配记录树节点头里的 key 区间不是它的位置规定罩的那一段",
            ));
        }
        if header.entries.is_empty() {
            return Err(violated(
                "分配记录树节点一条条目都没有（缺席的一段不写节点）",
            ));
        }
        if header.level == 0 {
            let first_new_record = records.len();
            for entry in &header.entries {
                let record = AllocationRecord::parse(entry).ok_or(
                    RecoveryFailure::EntryNarrowerThanItsFieldTable {
                        what: "分配记录",
                        entry_bytes: entry.len(),
                        field_table_bytes: usize::try_from(ALLOCATION_RECORD_BYTES).expect("20"),
                    },
                )?;
                if self.geometry.leaf_of_slot(record.device, record.slot) != node
                    || !self.geometry.record_fits_in_its_leaf(&record)
                {
                    return Err(RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
                        what: "分配记录不在它所在叶按位置罩的那一段里，或末槽越过叶的末槽",
                    });
                }
                records.push(record);
            }
            let keys_ascend = records[first_new_record..]
                .windows(2)
                .all(|pair| pair[0].sort_key() < pair[1].sort_key());
            if !keys_ascend {
                return Err(violated("分配记录树叶里的记录不按 key 严格递增"));
            }
            nodes.push((node, *pointer, bytes));
            return Ok(());
        }
        let internal_width =
            usize::try_from(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES).expect("96");
        if header.entry_width < internal_width {
            return Err(RecoveryFailure::EntryNarrowerThanItsFieldTable {
                what: "分配记录树内部条目",
                entry_bytes: header.entry_width,
                field_table_bytes: internal_width,
            });
        }
        let mut previous_child: Option<AllocationRecordTreeNode> = None;
        // 迭代上界是这个节点的条目数；跨轮携带的是上一个孩子（孩子要按位置严格递增）。
        for entry in &header.entries {
            let child = self
                .geometry
                .child_named_by_entry_key(node, &entry[..key_bytes_width()])
                .ok_or(violated(
                    "分配记录树内部条目的 key 不是这个节点里一个孩子那一段的起点",
                ))?;
            if previous_child.is_some_and(|previous| previous >= child) {
                return Err(violated("分配记录树内部节点的孩子不按位置严格递增"));
            }
            previous_child = Some(child);
            let mut reader = ByteReader::at(entry, key_bytes_width());
            let child_pointer = NodePointer::read_from(&mut reader);
            self.read_node_and_its_subtree(
                child,
                &child_pointer,
                read_node,
                units_seen,
                nodes,
                records,
            )?;
        }
        nodes.push((node, *pointer, bytes));
        Ok(())
    }
}

/// 从根指针往下读一整棵分配记录树（D8（核心索引结构） 已定项 14：挂载时分配记录树整棵读进挂载态，用户 K4），逐节点按位置核：
/// 根的层级是池几何定的那一层；每个节点头里的 key 区间是它的位置规定罩的那一段；内部条目的 key 是一个孩子那一段的起点、按位置严格递增；
/// 孩子的层级是父层级减一；节点不空；叶里的记录按 key 严格递增、每条都落在它所在叶里（末槽不越过叶的末槽）；
/// 同一个单元不被两条父条目引用。`judgement` 为 `EveryHeaderAgainstItsReference` 时另核头里的树 ID、fsid、诞生、出生序号。
/// 记录还要对这个池的几何判一遍（设备在池里、槽号在单元区里、跨度不越过单元区末尾、同盘不相交），由调用方做（`recovery`）。
///
/// # Errors
/// 节点读不到、解不开 ⇒ 读者交回的错或 `UnitMalformed`；条目窄于字段表 ⇒ `EntryNarrowerThanItsFieldTable`；
/// 记录不在它所在叶里 ⇒ `AllocationRecordOutsideThePoolGeometry`；别的核不过 ⇒ `InvariantViolated`（I-1.1 / I-1.2 / I-1.3 / I-1.4）。
#[allow(
    clippy::too_many_arguments,
    reason = "读一棵树要的：根指针、几何、树号、核到哪一步、根、fsid、读节点的口子，各自独立"
)]
pub fn read_allocation_record_tree(
    root_pointer: &NodePointer,
    geometry: &AllocationRecordTreeGeometry,
    tree: TreeIdentifier,
    judgement: AllocationRecordTreeHeaderJudgement,
    root: &RootRecord,
    expected_filesystem_identifier: u64,
    read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
) -> Result<AllocationRecordTreeReadFromDisk, RecoveryFailure> {
    let reading = AllocationRecordTreeReading {
        geometry,
        tree,
        judgement,
        root,
        expected_filesystem_identifier,
    };
    let mut nodes = Vec::new();
    let mut records = Vec::new();
    let mut units_seen = BTreeSet::new();
    reading.read_node_and_its_subtree(
        AllocationRecordTreeNode::Root,
        root_pointer,
        read_node,
        &mut units_seen,
        &mut nodes,
        &mut records,
    )?;
    nodes.sort_by_key(|(node, _, _)| *node);
    Ok(AllocationRecordTreeReadFromDisk { nodes, records })
}

/// 从根指针往下，读得出多少就认多少：交回走得到的每个节点的指针（根在内；读不出、解不开、位置对不上的节点它自己的指针照样交回，
/// 只是不再往它下面走）。影子账认一条被抛弃根引用着哪些单元时用它（`mount`）：这棵树的节点都是这条根引用的单元，
/// 少认一个，被抛弃根的那一个就不隔离、回退之后可能被发出去——同实例表那条链「认得出的都算上」。
#[must_use]
pub fn node_pointers_as_far_as_readable(
    root_pointer: &NodePointer,
    geometry: &AllocationRecordTreeGeometry,
    read_node: &mut dyn FnMut(&NodePointer) -> Option<Vec<u8>>,
) -> Vec<NodePointer> {
    let mut pointers = Vec::new();
    let mut units_seen = BTreeSet::new();
    let mut pending = vec![(AllocationRecordTreeNode::Root, *root_pointer)];
    // 迭代上界：每个节点至多进一次（按它第一条位置条目的槽认过的不再读），节点数有上界。
    while let Some((node, pointer)) = pending.pop() {
        if !units_seen.insert(pointer.locations[0].slot) {
            continue;
        }
        pointers.push(pointer);
        if geometry.level_of(node) == 0 {
            continue;
        }
        let Some(header) = read_node(&pointer).and_then(|bytes| parse_index_node(&bytes).ok())
        else {
            continue;
        };
        if header.level != geometry.level_of(node)
            || header.entry_width
                < usize::try_from(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES).expect("96")
        {
            continue;
        }
        for entry in &header.entries {
            if let Some(child) =
                geometry.child_named_by_entry_key(node, &entry[..key_bytes_width()])
            {
                let child_pointer =
                    NodePointer::read_from(&mut ByteReader::at(entry, key_bytes_width()));
                pending.push((child, child_pointer));
            }
        }
    }
    pointers
}

#[cfg(test)]
mod tests {
    use super::*;
    use singlefs_format::SLOT_BYTES;

    fn record(device: u32, slot: u64, span_slots: u16) -> AllocationRecord {
        AllocationRecord {
            device: DeviceIdentity(device),
            slot: SlotNumber(slot),
            span_slots,
            generation: CheckpointTxg(1),
            is_released: false,
        }
    }

    fn geometry_of_two_four_gibibyte_devices() -> AllocationRecordTreeGeometry {
        let slots = (4u64 << 30) / SLOT_BYTES;
        AllocationRecordTreeGeometry::of_devices(&[
            (DeviceIdentity(0), slots),
            (DeviceIdentity(1), slots),
        ])
    }

    /// 本地腿两次抽样一致的层数（判决 S1）：4 GiB 两盘 3 层——叶罩 812 槽、层级 1 罩 137228 槽，每盘 262144 槽要两格，两盘四格装进根。
    #[test]
    fn two_four_gibibyte_devices_give_a_tree_of_height_three() {
        let geometry = geometry_of_two_four_gibibyte_devices();
        assert_eq!(geometry.root_level(), 2);
        assert_eq!(geometry.height(), 3);
        assert_eq!(span_in_slots_at_level(1), 812 * 169);
    }

    /// 1 TiB 两盘 4 层（判决 S1 的 1 TiB 那一格）。
    #[test]
    fn two_one_tebibyte_devices_give_a_tree_of_height_four() {
        let slots = (1u64 << 40) / SLOT_BYTES;
        let geometry = AllocationRecordTreeGeometry::of_devices(&[
            (DeviceIdentity(0), slots),
            (DeviceIdentity(1), slots),
        ]);
        assert_eq!(geometry.height(), 4);
    }

    /// 盘上绝对槽数按整槽数（[`absolute_slot_count_of_device`]：盘尾不足一槽的零头不算）。每盘 812 × 84 = 68208 槽时两盘各 84 格、
    /// 168 ≤ 169，根在第 1 层；每盘字节数再多半个槽（不是 16384 的整数倍）根层级不变——按向上取整读会读成 68209 槽、两盘各 85 格、
    /// 170 > 169、根到第 2 层。对照：每盘多一个整槽就是第 2 层，这一格正好压在根层级的分界上。
    #[test]
    fn half_a_slot_at_the_end_of_each_device_does_not_move_the_root_level_across_its_boundary() {
        let slots_of_eighty_four_leaves = ALLOCATION_RECORD_TREE_LEAF_SLOTS * 84;
        let device_bytes_with_half_a_slot_at_the_end =
            slots_of_eighty_four_leaves * SLOT_BYTES + SLOT_BYTES / 2;
        let with_half_a_slot = AllocationRecordTreeGeometry::of_device_sizes_in_bytes(&[
            (DeviceIdentity(0), device_bytes_with_half_a_slot_at_the_end),
            (DeviceIdentity(1), device_bytes_with_half_a_slot_at_the_end),
        ]);
        assert_eq!(
            with_half_a_slot.root_level(),
            1,
            "盘尾那半个槽不算：两盘各 84 格，根在第 1 层"
        );
        let with_one_more_whole_slot = AllocationRecordTreeGeometry::of_devices(&[
            (DeviceIdentity(0), slots_of_eighty_four_leaves + 1),
            (DeviceIdentity(1), slots_of_eighty_four_leaves + 1),
        ]);
        assert_eq!(
            with_one_more_whole_slot.root_level(),
            2,
            "对照：每盘多一个整槽，两盘各 85 格装不进 169，根到第 2 层"
        );
    }

    /// 叶按绝对槽号划：槽 811 与 812 分在两片叶，第 k 片叶的 key 区间就是 `[(盘, k × 812), (盘, (k + 1) × 812 − 1)]`。
    #[test]
    fn leaves_split_the_slots_at_multiples_of_the_leaf_width() {
        let geometry = geometry_of_two_four_gibibyte_devices();
        let leaf_of = |slot: u64| geometry.leaf_of_slot(DeviceIdentity(1), SlotNumber(slot));
        assert_ne!(leaf_of(811), leaf_of(812));
        assert_eq!(leaf_of(812), leaf_of(1623));
        let (smallest, largest) = geometry.key_range_of(leaf_of(50176));
        assert_eq!(
            (smallest, largest),
            (
                allocation_record_key_bytes(DeviceIdentity(1), SlotNumber(61 * 812)),
                allocation_record_key_bytes(DeviceIdentity(1), SlotNumber(62 * 812 - 1))
            )
        );
    }

    /// 一个叶的记录变了：那片叶、它的父节点与根变，同一块盘上另一片叶与另一块盘上的整条路径不变。
    #[test]
    fn a_changed_record_changes_its_leaf_and_every_ancestor_only() {
        let geometry = geometry_of_two_four_gibibyte_devices();
        let before = vec![
            record(0, 50176, 2),
            record(0, 51000, 1),
            record(1, 50176, 2),
            record(1, 51000, 1),
        ];
        let mut after = before.clone();
        after[1].is_released = true;
        let changed = nodes_whose_contents_changed(&geometry, &before, &after);
        let leaf_of_the_changed_record =
            geometry.leaf_of_slot(DeviceIdentity(0), SlotNumber(51000));
        let mut expected = BTreeSet::new();
        expected.insert(leaf_of_the_changed_record);
        expected.insert(
            geometry
                .parent_of(leaf_of_the_changed_record)
                .expect("叶有父节点"),
        );
        expected.insert(AllocationRecordTreeNode::Root);
        assert_eq!(changed, expected);
    }

    /// 父节点里一条条目的 key 不在孩子那一层的格点上、或指到别的盘、或落在父节点那一段之外，都不是它的孩子。
    #[test]
    fn an_entry_key_off_the_grid_names_no_child() {
        let geometry = geometry_of_two_four_gibibyte_devices();
        let root = AllocationRecordTreeNode::Root;
        let level_one_start = span_in_slots_at_level(1);
        assert!(geometry
            .child_named_by_entry_key(
                root,
                &allocation_record_key_bytes(DeviceIdentity(0), SlotNumber(level_one_start))
            )
            .is_some());
        assert!(geometry
            .child_named_by_entry_key(
                root,
                &allocation_record_key_bytes(DeviceIdentity(0), SlotNumber(level_one_start + 2))
            )
            .is_none());
        assert!(geometry
            .child_named_by_entry_key(
                root,
                &allocation_record_key_bytes(DeviceIdentity(7), SlotNumber(0))
            )
            .is_none());
        let level_one = geometry
            .child_named_by_entry_key(
                root,
                &allocation_record_key_bytes(DeviceIdentity(0), SlotNumber(0)),
            )
            .expect("盘 0 第 0 个层级 1 节点");
        assert!(geometry
            .child_named_by_entry_key(
                level_one,
                &allocation_record_key_bytes(DeviceIdentity(1), SlotNumber(812))
            )
            .is_none());
    }
}
