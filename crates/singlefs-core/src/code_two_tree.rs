//! 多层码 2 树的结构规则（D8（核心索引结构） 已定项 11「多层码 2 树怎么长、怎么收」）：记账树与中央映射树共用这一份，
//! 不各写一套（已定项 14：一套 btree 实现管权威态的树与照分裂做的树）。
//!
//! 这里只管「这次发布之后树长什么样」：读上一版的形状（每个节点装哪些 key、父子关系）与这一版的 key 集合，
//! 算出这一版每个节点装哪些 key、它照抄上一版哪个节点还是这次重写、上一版哪些节点被换下。只读——不碰分配器、不发写、
//! 不装字节，交回拒绝时盘上逐字节不变。装字节、取落点在 `crate::transaction` 那一侧；从盘上读回一棵树在 [`read_code_two_tree`]。
//!
//! 规则逐条对应已定项 11：
//! ① 分裂出来的节点与别的单元落在同一段、整条根到叶的路径照常 COW：这里只把路径上的节点标成「这次重写」，写序由发布路径统一走；
//! ② 叶装不下时从中间切：插入之后条目数超过容量就切成两半，左半 ⌈n ÷ 2⌉ 条、右半其余；内部节点同一条；
//! ③ 收缩只摘空节点，根只剩一个孩子时降高一层：删到空的节点从父节点里摘掉，不做「低于一半合并」与借条目；
//!    根只剩一个孩子时孩子当根，孩子不重写；
//! ④ 分隔 key 跟着维护：插到最左分隔 key 之下时把它压低；切出来的右半在父节点里的分隔 key 取它自己的最小 key；
//! ⑤ 树高 = 根节点头里的层级 + 1（[`CodeTwoTreeShape::height`]；发布路径从根节点的字节现读，见 `crate::transaction`）。
//!
//! 节点的身份、层级、覆盖区间一律读块头自带的那几样，不另存旁路表：读回来的每个节点都按父条目核它的层级与区间
//! （[`read_code_two_tree`]）。

use std::collections::BTreeSet;

use singlefs_format::NODE_POINTER_BYTES;

use crate::address::{SlotNumber, TreeIdentifier};
use crate::bytes::{ByteReader, ByteWriter};
use crate::pointer::NodePointer;
use crate::recovery::RecoveryFailure;
use crate::root_record::RootRecord;
use crate::unit::{index_node_entry_capacity, parse_index_node, IndexNodeHeader};

/// 一棵码 2 树的 key 怎么比：各字段的字节宽，盘上小端；比较时逐字段按无符号整数、自左向右（D8（核心索引结构） 已定项 11），
/// 小端存储不构成 memcmp 序。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodeTwoKeyFieldWidths(&'static [usize]);

impl CodeTwoKeyFieldWidths {
    /// 记账条目的 key 22：统计量标签 2 + 树 ID 8 + 设备 4 + 代 8（D5（快照 / 空间记账机制） 已定项 5）。
    pub const ACCOUNTING: Self = Self(&[2, 8, 4, 8]);
    /// 中央映射 key 27：类标签 1 + 出生树 8 + 出生 txg 8 + 实例代号 4 + 尾段 6（D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10：
    /// 码 1 的尾段是事务号低 48 位；码 2 / 码 3 是出生序号 4 + 补零 2，按 6 字节整数比与按出生序号比同序）。
    pub const CENTRAL_MAPPING: Self = Self(&[1, 8, 8, 4, 6]);

    /// key 宽：各字段宽之和。
    #[must_use]
    pub fn key_width_in_bytes(self) -> usize {
        self.0.iter().sum()
    }

    fn fields_in_comparison_order(self, key_bytes: &[u8]) -> Vec<u64> {
        assert_eq!(
            key_bytes.len(),
            self.key_width_in_bytes(),
            "key 的字节数等于这棵树的 key 宽：调用方从条目里切前 key 宽 个字节（D8 已定项 11）"
        );
        let mut offset_in_bytes = 0;
        self.0
            .iter()
            .map(|field_width_in_bytes| {
                let mut little_endian = [0u8; 8];
                little_endian[..*field_width_in_bytes].copy_from_slice(
                    &key_bytes[offset_in_bytes..offset_in_bytes + field_width_in_bytes],
                );
                offset_in_bytes += field_width_in_bytes;
                u64::from_le_bytes(little_endian)
            })
            .collect()
    }
}

/// 一把码 2 key：盘上的字节与按字段读出来的无符号整数。全序只看后者（[`CodeTwoKeyFieldWidths`]）；
/// 同一棵树里字段相等的两把 key 字节也相等，所以派生的全序先比字段、再比字节，与只比字段同序。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CodeTwoTreeKey {
    fields_in_comparison_order: Vec<u64>,
    bytes: Vec<u8>,
}

impl CodeTwoTreeKey {
    /// # Panics
    /// `key_bytes` 的长度不是这棵树的 key 宽：调用方按 key 宽切出来的，不等说明切错了。
    #[must_use]
    pub fn new(key_bytes: &[u8], field_widths: CodeTwoKeyFieldWidths) -> Self {
        Self {
            fields_in_comparison_order: field_widths.fields_in_comparison_order(key_bytes),
            bytes: key_bytes.to_vec(),
        }
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// 一个码 2 节点最多装几条条目：叶按这棵树的叶条目宽算，内部节点按内部条目宽算（D8（核心索引结构） 已定项 11：
/// 内部节点的条目 = 本树 key + 子指针 86；记账树 108、中央映射树 113）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodeTwoTreeNodeCapacity {
    pub leaf_entries: usize,
    pub internal_entries: usize,
}

impl CodeTwoTreeNodeCapacity {
    /// 格式算出来的容量：(16384 − 含预留位的头) ÷ 条目宽（`unit::index_node_entry_capacity`）。
    #[must_use]
    pub fn of_the_node_format(key_width_in_bytes: usize, leaf_entry_width_in_bytes: usize) -> Self {
        Self {
            leaf_entries: index_node_entry_capacity(key_width_in_bytes, leaf_entry_width_in_bytes),
            internal_entries: index_node_entry_capacity(
                key_width_in_bytes,
                internal_entry_width_in_bytes(key_width_in_bytes),
            ),
        }
    }
}

/// 内部节点的条目宽：本树 key + 子指针 86（D8（核心索引结构） 已定项 11）。
#[must_use]
pub fn internal_entry_width_in_bytes(key_width_in_bytes: usize) -> usize {
    key_width_in_bytes + usize::try_from(NODE_POINTER_BYTES).expect("86")
}

/// 一条内部节点条目：分隔 key 打头（D8（核心索引结构） 已定项 11「条目里 key 一律是条目的前 key 宽 个字节」），后跟指向孩子的节点指针。
#[must_use]
pub fn build_internal_entry(separator_key: &[u8], child: NodePointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(internal_entry_width_in_bytes(separator_key.len()));
    writer.put(separator_key);
    child.write_to(&mut writer);
    writer.into_bytes()
}

/// 解一条内部节点条目：条目宽是节点头里的**盘上字段**，窄于 key 宽 + 86 时交回 `None`（调用方报「条目窄于字段表」），
/// 这里是盘上字节进字段表的边界。
#[must_use]
pub fn parse_internal_entry(
    entry: &[u8],
    key_width_in_bytes: usize,
) -> Option<(Vec<u8>, NodePointer)> {
    if entry.len() < internal_entry_width_in_bytes(key_width_in_bytes) {
        return None;
    }
    let mut reader = ByteReader::at(entry, key_width_in_bytes);
    Some((
        entry[..key_width_in_bytes].to_vec(),
        NodePointer::read_from(&mut reader),
    ))
}

/// 一个码 2 节点在一版树里的位置：层级（0 是叶）与同层从左数第几个（按 key 升序）。派生的全序（层级升序、同层按 key 升序）
/// 就是这棵树在一次发布里的 bump 次序与出生序号的发号次序：树内先叶后根、同层按 key 升序（D3（空间分配） 已定项 10 ⑤、
/// D19（块指针的结构与宽度预算） 已定项 9），根在最高那一层、排最末。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CodeTwoTreeNodePosition {
    pub level: u8,
    pub index_in_level: u32,
}

/// 一个内部节点的一条条目在形状里的样子：分隔 key 与它指着的孩子。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeTwoTreeChild {
    pub separator_key: CodeTwoTreeKey,
    pub child: CodeTwoTreeNodePosition,
}

/// 一个节点装的东西：叶装条目的 key（按 key 升序），内部节点装孩子（按分隔 key 升序）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CodeTwoTreeNodeContents {
    Leaf { keys: Vec<CodeTwoTreeKey> },
    Internal { children: Vec<CodeTwoTreeChild> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeTwoTreeShapeNode {
    pub position: CodeTwoTreeNodePosition,
    pub contents: CodeTwoTreeNodeContents,
}

/// 一版树的形状：节点按 bump 次序（[`CodeTwoTreeNodePosition`] 的全序），根在最末。还没建过的树没有节点。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CodeTwoTreeShape {
    nodes: Vec<CodeTwoTreeShapeNode>,
}

impl CodeTwoTreeShape {
    #[must_use]
    pub fn nodes(&self) -> &[CodeTwoTreeShapeNode] {
        &self.nodes
    }

    /// 根：bump 次序里的最后一个节点。还没建过的树交回 `None`。
    #[must_use]
    pub fn root(&self) -> Option<&CodeTwoTreeShapeNode> {
        self.nodes.last()
    }

    /// 树高 = 根的层级 + 1（D8（核心索引结构） 已定项 11 ⑤）；还没建过的树是 0。
    #[must_use]
    pub fn height(&self) -> u64 {
        self.root()
            .map_or(0, |root| u64::from(root.position.level) + 1)
    }

    #[must_use]
    pub fn node(&self, position: CodeTwoTreeNodePosition) -> Option<&CodeTwoTreeShapeNode> {
        self.nodes
            .binary_search_by_key(&position, |node| node.position)
            .ok()
            .map(|index| &self.nodes[index])
    }

    /// 这个位置是不是根。
    #[must_use]
    pub fn is_root(&self, position: CodeTwoTreeNodePosition) -> bool {
        self.root().is_some_and(|root| root.position == position)
    }

    /// 叶里的全部 key，从左到右。
    #[must_use]
    pub fn keys_in_order(&self) -> Vec<&CodeTwoTreeKey> {
        self.nodes
            .iter()
            .filter_map(|node| match &node.contents {
                CodeTwoTreeNodeContents::Leaf { keys } => Some(keys.iter()),
                CodeTwoTreeNodeContents::Internal { .. } => None,
            })
            .flatten()
            .collect()
    }

    /// 一个节点的子树里最小与最大的 key（D18（块里携带什么信息） 已定项 2：码 2 头的 key 区间取子树覆盖区间）。
    /// 空的叶（还没装过条目的根）交回 `None`。
    ///
    /// # Panics
    /// 形状里的子引用指到了不存在的位置：形状由 [`plan_the_tree_after_this_publish`] 或 [`read_code_two_tree`] 造出来，
    /// 两处都按层级逐层建引用，指不到说明造它的那一方写错了。
    #[must_use]
    pub fn key_range_of_the_subtree(
        &self,
        position: CodeTwoTreeNodePosition,
    ) -> Option<(&CodeTwoTreeKey, &CodeTwoTreeKey)> {
        let mut leftmost = position;
        let smallest = loop {
            match &self
                .node(leftmost)
                .expect("子引用指着形状里的节点")
                .contents
            {
                CodeTwoTreeNodeContents::Leaf { keys } => break keys.first()?,
                CodeTwoTreeNodeContents::Internal { children } => {
                    leftmost = children.first()?.child;
                }
            }
        };
        let mut rightmost = position;
        let largest = loop {
            match &self
                .node(rightmost)
                .expect("子引用指着形状里的节点")
                .contents
            {
                CodeTwoTreeNodeContents::Leaf { keys } => break keys.last()?,
                CodeTwoTreeNodeContents::Internal { children } => {
                    rightmost = children.last()?.child;
                }
            }
        };
        Some((smallest, largest))
    }
}

/// 这一版的一个节点从哪来：照抄上一版的某个节点（字节、落点、指针一个不动），或这次重写。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeTwoTreeNodeOrigin {
    CarriedFrom(CodeTwoTreeNodePosition),
    RewrittenThisPublish,
}

/// 这次发布之后树长什么样（[`plan_the_tree_after_this_publish`] 交回）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeTwoTreePlan {
    pub shape: CodeTwoTreeShape,
    /// 与 `shape.nodes()` 同序。
    pub origins: Vec<CodeTwoTreeNodeOrigin>,
    /// 上一版里这次被换下的节点（没被照抄进这一版的），按上一版的 bump 次序。
    pub replaced_previous_nodes: Vec<CodeTwoTreeNodePosition>,
}

impl CodeTwoTreePlan {
    /// 这次重写的节点，按 bump 次序（= 这棵树里出生序号的发号次序）。
    #[must_use]
    pub fn rewritten_positions(&self) -> Vec<CodeTwoTreeNodePosition> {
        self.shape
            .nodes()
            .iter()
            .zip(&self.origins)
            .filter(|(_, origin)| **origin == CodeTwoTreeNodeOrigin::RewrittenThisPublish)
            .map(|(node, _)| node.position)
            .collect()
    }

    /// 这个位置上的节点从哪来。
    ///
    /// # Panics
    /// 这个位置不在这一版里：调用方从同一份计划里取的位置。
    #[must_use]
    pub fn origin_of(&self, position: CodeTwoTreeNodePosition) -> CodeTwoTreeNodeOrigin {
        let index = self
            .shape
            .nodes()
            .binary_search_by_key(&position, |node| node.position)
            .expect("位置取自同一份计划");
        self.origins[index]
    }
}

/// 这次之后的树算不出来，在任何落盘动作之前交回。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeTwoTreeRefusal {
    /// 这次之后树要长到 257 层：码 2 头里的层级是 1 字节（D18（块里携带什么信息） 已定项 18 偏移 50），写不下。
    /// 这是多层码 2 树唯一的结构上限（D19（块指针的结构与宽度预算） 已定项 5 的映射树容量准入在多层之后就是它）。
    HeightBeyondTheLevelField,
    /// 上一版的形状里，按分隔 key 从根往下走，走不到装着某把 key 的那片叶：形状是从盘上读来的（重建出来的上一版），
    /// 分隔 key 与孩子的区间对不上时删不掉它。读回来的形状按父条目核过区间（[`read_code_two_tree`]），走到这里说明核漏了；
    /// 不断言，交回让这次发布整次不做。
    PreviousShapeRoutesAKeyAwayFromTheLeafHoldingIt,
}

/// 规划中的一个节点：`carried_from` 是 `Some` ⟺ 从上一版那个位置照抄、这次没被碰过。
#[derive(Clone, Debug)]
enum PlanningNode {
    Leaf {
        keys: Vec<CodeTwoTreeKey>,
        carried_from: Option<CodeTwoTreeNodePosition>,
    },
    Internal {
        level: u8,
        children: Vec<PlanningChild>,
        carried_from: Option<CodeTwoTreeNodePosition>,
    },
}

#[derive(Clone, Debug)]
struct PlanningChild {
    separator_key: CodeTwoTreeKey,
    node: PlanningNode,
}

impl PlanningNode {
    fn empty_leaf() -> Self {
        PlanningNode::Leaf {
            keys: Vec::new(),
            carried_from: None,
        }
    }

    fn level(&self) -> u8 {
        match self {
            PlanningNode::Leaf { .. } => 0,
            PlanningNode::Internal { level, .. } => *level,
        }
    }

    fn entry_count(&self) -> usize {
        match self {
            PlanningNode::Leaf { keys, .. } => keys.len(),
            PlanningNode::Internal { children, .. } => children.len(),
        }
    }

    /// 这次碰过它：路径上的节点整条 COW（D8（核心索引结构） 已定项 11 ①）。
    fn mark_rewritten(&mut self) {
        match self {
            PlanningNode::Leaf { carried_from, .. }
            | PlanningNode::Internal { carried_from, .. } => *carried_from = None,
        }
    }

    fn smallest_key(&self) -> Option<&CodeTwoTreeKey> {
        match self {
            PlanningNode::Leaf { keys, .. } => keys.first(),
            PlanningNode::Internal { children, .. } => children.first()?.node.smallest_key(),
        }
    }
}

/// 从上一版的形状建规划树：每个节点都标成「照抄自原位置」。
fn planning_node_of(shape: &CodeTwoTreeShape, position: CodeTwoTreeNodePosition) -> PlanningNode {
    match &shape
        .node(position)
        .expect("形状里的子引用指着形状里的节点（造形状的一方逐层建引用）")
        .contents
    {
        CodeTwoTreeNodeContents::Leaf { keys } => PlanningNode::Leaf {
            keys: keys.clone(),
            carried_from: Some(position),
        },
        CodeTwoTreeNodeContents::Internal { children } => PlanningNode::Internal {
            level: position.level,
            children: children
                .iter()
                .map(|child| PlanningChild {
                    separator_key: child.separator_key.clone(),
                    node: planning_node_of(shape, child.child),
                })
                .collect(),
            carried_from: Some(position),
        },
    }
}

/// 按分隔 key 找孩子（插入用）：最后一个分隔 key ≤ key 的那一个；key 比第一个分隔 key 还小时落到第 0 个孩子，
/// 并把第 0 个分隔 key 压低到 key（D8（核心索引结构） 已定项 11 ④）。
fn route_for_insertion(children: &mut [PlanningChild], key: &CodeTwoTreeKey) -> usize {
    match children
        .iter()
        .rposition(|child| child.separator_key <= *key)
    {
        Some(index) => index,
        None => {
            children[0].separator_key = key.clone();
            0
        }
    }
}

/// 按分隔 key 找孩子（删除用）：同上，不改分隔 key。
fn route_for_deletion(children: &[PlanningChild], key: &CodeTwoTreeKey) -> usize {
    children
        .iter()
        .rposition(|child| child.separator_key <= *key)
        .unwrap_or(0)
}

/// 从中间切：左半留 ⌈n ÷ 2⌉ 条，交回右半与它在父节点里的分隔 key（它自己的最小 key）。
fn split_in_the_middle(node: &mut PlanningNode) -> PlanningChild {
    match node {
        PlanningNode::Leaf { keys, .. } => {
            let right_keys = keys.split_off(keys.len().div_ceil(2));
            PlanningChild {
                separator_key: right_keys[0].clone(),
                node: PlanningNode::Leaf {
                    keys: right_keys,
                    carried_from: None,
                },
            }
        }
        PlanningNode::Internal {
            level, children, ..
        } => {
            let right_children = children.split_off(children.len().div_ceil(2));
            PlanningChild {
                separator_key: right_children[0].separator_key.clone(),
                node: PlanningNode::Internal {
                    level: *level,
                    children: right_children,
                    carried_from: None,
                },
            }
        }
    }
}

/// 插一把 key；节点装不下时切开，交回右半（由父节点接上）。路径上每个节点都标成重写。
fn insert_below(
    node: &mut PlanningNode,
    key: CodeTwoTreeKey,
    capacity: CodeTwoTreeNodeCapacity,
) -> Option<PlanningChild> {
    node.mark_rewritten();
    match node {
        PlanningNode::Leaf { keys, .. } => {
            let insertion_index = keys.partition_point(|existing| *existing < key);
            assert!(
                keys.get(insertion_index) != Some(&key),
                "插入的 key 不在树里：插入集合 = 这一版的 key 减上一版的 key"
            );
            keys.insert(insertion_index, key);
            if keys.len() > capacity.leaf_entries {
                Some(split_in_the_middle(node))
            } else {
                None
            }
        }
        PlanningNode::Internal { children, .. } => {
            let child_index = route_for_insertion(children, &key);
            if let Some(right_half) = insert_below(&mut children[child_index].node, key, capacity) {
                children.insert(child_index + 1, right_half);
            }
            if children.len() > capacity.internal_entries {
                Some(split_in_the_middle(node))
            } else {
                None
            }
        }
    }
}

/// 在根上插一把 key：根装不下时切开、长出新根，树高 + 1。
fn insert_at_the_root(
    root: &mut PlanningNode,
    key: CodeTwoTreeKey,
    capacity: CodeTwoTreeNodeCapacity,
) -> Result<(), CodeTwoTreeRefusal> {
    let Some(right_half) = insert_below(root, key, capacity) else {
        return Ok(());
    };
    let new_root_level = root
        .level()
        .checked_add(1)
        .ok_or(CodeTwoTreeRefusal::HeightBeyondTheLevelField)?;
    let left_half = std::mem::replace(root, PlanningNode::empty_leaf());
    let left_separator = left_half
        .smallest_key()
        .expect("刚装满、切开过的节点左半不空")
        .clone();
    *root = PlanningNode::Internal {
        level: new_root_level,
        children: vec![
            PlanningChild {
                separator_key: left_separator,
                node: left_half,
            },
            right_half,
        ],
        carried_from: None,
    };
    Ok(())
}

/// 删一把 key；删空的孩子从父节点里摘掉（只摘空节点，D8（核心索引结构） 已定项 11 ③）。交回找没找到。
fn delete_below(node: &mut PlanningNode, key: &CodeTwoTreeKey) -> bool {
    match node {
        PlanningNode::Leaf { keys, .. } => match keys.binary_search(key) {
            Ok(index) => {
                keys.remove(index);
                node.mark_rewritten();
                true
            }
            Err(_) => false,
        },
        PlanningNode::Internal { children, .. } => {
            let child_index = route_for_deletion(children, key);
            if !delete_below(&mut children[child_index].node, key) {
                return false;
            }
            if children[child_index].node.entry_count() == 0 {
                children.remove(child_index);
            }
            node.mark_rewritten();
            true
        }
    }
}

/// 根上删一把 key，删完之后根空了就变回空叶，根只剩一个孩子就让孩子当根（降高一层，孩子不重写），直到不再成立。
fn delete_at_the_root(
    root: &mut PlanningNode,
    key: &CodeTwoTreeKey,
) -> Result<(), CodeTwoTreeRefusal> {
    if !delete_below(root, key) {
        return Err(CodeTwoTreeRefusal::PreviousShapeRoutesAKeyAwayFromTheLeafHoldingIt);
    }
    // 迭代上界：每一轮树高减一，至多根的层级那么多轮。
    loop {
        let only_child = match root {
            PlanningNode::Internal { children, .. } if children.is_empty() => {
                *root = PlanningNode::empty_leaf();
                return Ok(());
            }
            PlanningNode::Internal { children, .. } if children.len() == 1 => {
                children.pop().expect("刚判过恰一个孩子").node
            }
            PlanningNode::Internal { .. } | PlanningNode::Leaf { .. } => return Ok(()),
        };
        *root = only_child;
    }
}

/// 把规划树按层摊平：后序、从左到右，每一层里的节点就按 key 升序排好（同层左边的子树先走完）。
fn flatten(
    node: PlanningNode,
    nodes_by_level: &mut Vec<Vec<(CodeTwoTreeNodeContents, CodeTwoTreeNodeOrigin)>>,
) -> CodeTwoTreeNodePosition {
    let level = node.level();
    let (contents, carried_from) = match node {
        PlanningNode::Leaf { keys, carried_from } => {
            (CodeTwoTreeNodeContents::Leaf { keys }, carried_from)
        }
        PlanningNode::Internal {
            children,
            carried_from,
            ..
        } => {
            let children = children
                .into_iter()
                .map(|child| CodeTwoTreeChild {
                    separator_key: child.separator_key,
                    child: flatten(child.node, nodes_by_level),
                })
                .collect();
            (CodeTwoTreeNodeContents::Internal { children }, carried_from)
        }
    };
    let level_index = usize::from(level);
    if nodes_by_level.len() <= level_index {
        nodes_by_level.resize_with(level_index + 1, Vec::new);
    }
    let index_in_level = u32::try_from(nodes_by_level[level_index].len())
        .expect("一层的节点数装得进 u32：条目数有上界");
    nodes_by_level[level_index].push((
        contents,
        match carried_from {
            Some(previous_position) => CodeTwoTreeNodeOrigin::CarriedFrom(previous_position),
            None => CodeTwoTreeNodeOrigin::RewrittenThisPublish,
        },
    ));
    CodeTwoTreeNodePosition {
        level,
        index_in_level,
    }
}

/// 这次发布之后树长什么样：上一版有、这一版没有的 key 先删（按 key 升序），这一版有、上一版没有的 key 再插（按 key 升序）。
/// 先删后插：一次发布里同一个 key 的中间状态不落叶（D8（核心索引结构） 已定项 13），这里只决定这一版的形状；
/// 先删让删空的节点先摘掉、新 key 再按分隔 key 找落处，树不因一次发布里的换号而先长后缩。
///
/// # Errors
/// 这次之后树要长到 257 层 ⇒ `HeightBeyondTheLevelField`；上一版的形状按分隔 key 走不到它自己叶里的 key ⇒
/// `PreviousShapeRoutesAKeyAwayFromTheLeafHoldingIt`。两样都在任何落盘动作之前交回。
///
/// # Panics
/// 这一版一把 key 都没有：记账树每次发布恒有池级 3 行，中央映射树恒有分配记录树、记账树与 inode 树根的条目，
/// 两棵树都不会删到空（调用方给的是这两棵树）。
pub fn plan_the_tree_after_this_publish(
    previous: &CodeTwoTreeShape,
    keys_after_this_publish: &BTreeSet<CodeTwoTreeKey>,
    capacity: CodeTwoTreeNodeCapacity,
) -> Result<CodeTwoTreePlan, CodeTwoTreeRefusal> {
    assert!(
        !keys_after_this_publish.is_empty(),
        "记账树恒有池级 3 行、中央映射树恒有分配记录树、记账树与 inode 树根的条目：这一版的 key 不会是空集"
    );
    assert!(
        capacity.leaf_entries >= 1 && capacity.internal_entries >= 2,
        "叶至少装 1 条、内部节点至少装 2 个孩子：切出来的两半才都不空、根分裂才装得下两个孩子"
    );
    let previous_keys: BTreeSet<CodeTwoTreeKey> =
        previous.keys_in_order().into_iter().cloned().collect();
    let mut root = match previous.root() {
        Some(previous_root) => planning_node_of(previous, previous_root.position),
        None => PlanningNode::empty_leaf(),
    };
    for deleted in previous_keys.difference(keys_after_this_publish) {
        delete_at_the_root(&mut root, deleted)?;
    }
    for inserted in keys_after_this_publish.difference(&previous_keys) {
        insert_at_the_root(&mut root, inserted.clone(), capacity)?;
    }
    let mut nodes_by_level = Vec::new();
    flatten(root, &mut nodes_by_level);
    let mut nodes = Vec::new();
    let mut origins = Vec::new();
    for (level, nodes_of_this_level) in nodes_by_level.into_iter().enumerate() {
        for (index_in_level, (contents, origin)) in nodes_of_this_level.into_iter().enumerate() {
            nodes.push(CodeTwoTreeShapeNode {
                position: CodeTwoTreeNodePosition {
                    level: u8::try_from(level).expect("层级是 1 字节：根分裂时判过"),
                    index_in_level: u32::try_from(index_in_level).expect("摊平时判过"),
                },
                contents,
            });
            origins.push(origin);
        }
    }
    let carried: BTreeSet<CodeTwoTreeNodePosition> = origins
        .iter()
        .filter_map(|origin| match origin {
            CodeTwoTreeNodeOrigin::CarriedFrom(previous_position) => Some(*previous_position),
            CodeTwoTreeNodeOrigin::RewrittenThisPublish => None,
        })
        .collect();
    let replaced_previous_nodes = previous
        .nodes()
        .iter()
        .map(|node| node.position)
        .filter(|position| !carried.contains(position))
        .collect();
    Ok(CodeTwoTreePlan {
        shape: CodeTwoTreeShape { nodes },
        origins,
        replaced_previous_nodes,
    })
}

/// 一版树在内存里的样子：形状加每个节点这一版的指针（与 `shape.nodes()` 同序）。节点的字节住
/// `TransactionOutput::units` 里它那个角色的单元（`transaction::TransactionUnit` 的记账树 / 中央映射树两族）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CodeTwoTreeVersion {
    pub shape: CodeTwoTreeShape,
    pub pointers: Vec<NodePointer>,
}

impl CodeTwoTreeVersion {
    /// 这个位置上的节点这一版的指针。
    ///
    /// # Panics
    /// 位置不在这一版里：调用方从同一份形状里取的位置。
    #[must_use]
    pub fn pointer_of(&self, position: CodeTwoTreeNodePosition) -> NodePointer {
        let index = self
            .shape
            .nodes()
            .binary_search_by_key(&position, |node| node.position)
            .expect("位置取自同一份形状");
        self.pointers[index]
    }

    /// 根的指针（树表条目或根记录里写的那一条）。
    ///
    /// # Panics
    /// 树还没建过：带文件的一版里记账树与中央映射树恒有根。
    #[must_use]
    pub fn root_pointer(&self) -> NodePointer {
        *self
            .pointers
            .last()
            .expect("带文件的一版里记账树与中央映射树恒有根")
    }

    /// 节点数。
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.shape.nodes().len()
    }
}

/// 从盘上读一棵多层码 2 树时对它的期望：哪棵树、key 怎么比，内部条目窄于 key 宽 + 86 时报错叫它什么。
#[derive(Clone, Copy, Debug)]
pub struct CodeTwoTreeReadExpectation {
    pub tree: TreeIdentifier,
    pub field_widths: CodeTwoKeyFieldWidths,
    pub internal_entry_name: &'static str,
}

/// 从盘上读回来的一棵多层码 2 树：形状、每个节点的指针与字节（与 `version.shape.nodes()` 同序），与叶里的全部条目（按 key 升序）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeTwoTreeReadFromDisk {
    pub version: CodeTwoTreeVersion,
    pub node_bytes: Vec<Vec<u8>>,
    pub leaf_entries_in_key_order: Vec<Vec<u8>>,
}

/// 读回来还没摊平的一个节点。
struct NodeReadFromDisk {
    pointer: NodePointer,
    bytes: Vec<u8>,
    header: IndexNodeHeader,
    children: Vec<(CodeTwoTreeKey, NodeReadFromDisk)>,
}

/// 从盘上读一棵多层码 2 树时核到哪一步。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeTwoTreeHeaderJudgement {
    /// 冷启动走读与挂载态打开：每个节点的头都按指着它的指针与父条目核（树 ID、key 宽、出生身份、fsid、层级、区间、分隔 key），
    /// 与走读核别的树根同一套口径、同一组判定名（根那一个节点的报错逐字沿用 `recovery` 核树根的那几句）。
    EveryHeaderAgainstItsReference,
    /// 从盘上重建上一版（`recovery::rebuild_version`）：只核把节点拼成一棵树非核不可的那几样——层级逐层减一、节点不空、
    /// 内部条目不窄于 key 宽 + 86、同一个节点不被两条父条目引用；头里的出生身份、区间与分隔 key 不核，与重建路径读别的树同一个口径
    /// （解得开就收，叶条目窄于字段表由解条目的一方报）。分隔 key 与孩子对不上时，下一次发布按它删 key 删不掉，由规划那一步交回
    /// `CodeTwoTreeRefusal::PreviousShapeRoutesAKeyAwayFromTheLeafHoldingIt`。
    OnlyWhatTheShapeNeeds,
}

fn violated(detail: &'static str) -> RecoveryFailure {
    RecoveryFailure::InvariantViolated {
        invariant: "I-1.1",
        detail,
    }
}

/// 核一个节点的头：只有 `EveryHeaderAgainstItsReference` 才走到这里。根（`is_root`）的报错逐字沿用 `recovery` 核树根的那几句，
/// 读路径上几种坏法报的成员因此不随树长没长多层而变。
fn judge_the_header_against_its_reference(
    header: &IndexNodeHeader,
    pointer: &NodePointer,
    expectation: &CodeTwoTreeReadExpectation,
    root: &RootRecord,
    expected_filesystem_identifier: u64,
    is_root: bool,
) -> Result<(), RecoveryFailure> {
    let detail = |of_the_root: &'static str, of_a_node_below_the_root: &'static str| {
        if is_root {
            of_the_root
        } else {
            of_a_node_below_the_root
        }
    };
    if header.tree != expectation.tree {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.3",
            detail: detail(
                "根头里的树 ID 与树表不符",
                "码 2 节点头里的树 ID 与引用它的树不符",
            ),
        });
    }
    if header.key_width != expectation.field_widths.key_width_in_bytes() {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "E142 走读同款",
            detail: detail(
                "根自述 key 宽与树的种类不符",
                "码 2 节点自述 key 宽与树的种类不符",
            ),
        });
    }
    if header.birth_txg > root.checkpoint_txg || header.instance > root.instance {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.2",
            detail: detail("树根诞生于根之后", "码 2 节点诞生于根之后"),
        });
    }
    if header.filesystem_identifier != expected_filesystem_identifier {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.4",
            detail: detail("树根 fsid 不符", "码 2 节点 fsid 不符"),
        });
    }
    if header.birth_sequence != pointer.birth_sequence {
        return Err(RecoveryFailure::InvariantViolated {
            invariant: "I-1.2",
            detail: detail("树根出生序号与指针不符", "码 2 节点出生序号与指针不符"),
        });
    }
    if header.level == 0 {
        let key_width = expectation.field_widths.key_width_in_bytes();
        let first_and_last_hold = match (header.entries.first(), header.entries.last()) {
            (Some(first), Some(last)) => {
                first[..key_width] == header.smallest_key[..]
                    && last[..key_width] == header.largest_key[..]
            }
            (None, _) | (_, None) => true,
        };
        if !first_and_last_hold {
            return Err(violated(detail(
                "根 key 区间与条目不符",
                "叶的 key 区间不是首末两条条目的 key",
            )));
        }
    }
    Ok(())
}

/// 从根指针往下读一棵多层码 2 树（记账树、中央映射树），节点的身份、层级、覆盖区间只信它自己头里写的那几样，逐个按父条目核
/// （核到哪一步看 `judgement`，[`CodeTwoTreeHeaderJudgement`]）：
/// - 头：树 ID 是这棵树（I-1.3）、key 宽是这棵树的、fsid 是本池（I-1.4）、诞生不晚于根、出生序号与指着它的指针相同（I-1.2）；
/// - 层级：孩子 = 父 − 1（I-1.1：层级是索引节点身份的一段），层级 0 是叶；
/// - 区间：叶的 key 区间是首末两条条目的 key；内部节点的是子树覆盖区间 [第一个孩子的最小 key, 最后一个孩子的最大 key]
///   （D18（块里携带什么信息） 已定项 2）；第 i 个孩子的区间落在父条目给的那一段里：分隔 key_i ≤ 孩子头里的最小 key，
///   且孩子头里的最大 key < 分隔 key_{i+1}（D8（核心索引结构） 已定项 11 ④ 的两条不等式）；条目按 key 严格递增；
/// - 两种核法都判：节点不空、同一个单元不被两条父条目引用（走成 DAG 就按层级指数放大读量）。
///
/// `read_node` 按指针读一个节点的字节：不进映射的树（中央映射树）只按位置条目读，进映射的树（记账树）走读时提示读不出经映射回退。
///
/// # Errors
/// 节点读不到、解不开 ⇒ 读者交回的错或 `UnitMalformed`；内部条目窄于 key 宽 + 86 ⇒ `EntryNarrowerThanItsFieldTable`；
/// 上面任一条核不过 ⇒ `InvariantViolated`（I-1.1 / I-1.2 / I-1.3 / I-1.4 / E142 走读同款）。
pub(crate) fn read_code_two_tree(
    root_pointer: &NodePointer,
    expectation: &CodeTwoTreeReadExpectation,
    judgement: CodeTwoTreeHeaderJudgement,
    root: &RootRecord,
    expected_filesystem_identifier: u64,
    read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
) -> Result<CodeTwoTreeReadFromDisk, RecoveryFailure> {
    let mut units_seen: BTreeSet<SlotNumber> = BTreeSet::new();
    let reading = TreeReading {
        expectation,
        judgement,
        root,
        expected_filesystem_identifier,
    };
    let root_node =
        reading.read_node_and_its_subtree(root_pointer, None, read_node, &mut units_seen)?;
    let mut nodes_by_level: Vec<Vec<(CodeTwoTreeNodeContents, NodePointer, Vec<u8>)>> = Vec::new();
    let mut leaf_entries_in_key_order = Vec::new();
    flatten_read(
        root_node,
        expectation.field_widths,
        &mut nodes_by_level,
        &mut leaf_entries_in_key_order,
    );
    let mut nodes = Vec::new();
    let mut pointers = Vec::new();
    let mut node_bytes = Vec::new();
    for (level, nodes_of_this_level) in nodes_by_level.into_iter().enumerate() {
        for (index_in_level, (contents, pointer, bytes)) in
            nodes_of_this_level.into_iter().enumerate()
        {
            nodes.push(CodeTwoTreeShapeNode {
                position: CodeTwoTreeNodePosition {
                    level: u8::try_from(level).expect("层级来自 1 字节的头字段"),
                    index_in_level: u32::try_from(index_in_level).expect("一层的节点数装得进 u32"),
                },
                contents,
            });
            pointers.push(pointer);
            node_bytes.push(bytes);
        }
    }
    Ok(CodeTwoTreeReadFromDisk {
        version: CodeTwoTreeVersion {
            shape: CodeTwoTreeShape { nodes },
            pointers,
        },
        node_bytes,
        leaf_entries_in_key_order,
    })
}

/// 读一棵树时一路不变的那几样。
struct TreeReading<'reading> {
    expectation: &'reading CodeTwoTreeReadExpectation,
    judgement: CodeTwoTreeHeaderJudgement,
    root: &'reading RootRecord,
    expected_filesystem_identifier: u64,
}

impl TreeReading<'_> {
    fn judges_every_header(&self) -> bool {
        match self.judgement {
            CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference => true,
            CodeTwoTreeHeaderJudgement::OnlyWhatTheShapeNeeds => false,
        }
    }

    fn key(&self, key_bytes: &[u8]) -> CodeTwoTreeKey {
        CodeTwoTreeKey::new(key_bytes, self.expectation.field_widths)
    }

    fn read_node_and_its_subtree(
        &self,
        pointer: &NodePointer,
        expected_level: Option<u8>,
        read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
        units_seen: &mut BTreeSet<SlotNumber>,
    ) -> Result<NodeReadFromDisk, RecoveryFailure> {
        if !units_seen.insert(pointer.locations[0].slot) {
            return Err(violated("同一个码 2 节点被两条父条目引用"));
        }
        let bytes = read_node(pointer)?;
        let header = parse_index_node(&bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
            what: if expected_level.is_none() {
                "树根节点"
            } else {
                "多层码 2 树的节点"
            },
        })?;
        if self.judges_every_header() {
            judge_the_header_against_its_reference(
                &header,
                pointer,
                self.expectation,
                self.root,
                self.expected_filesystem_identifier,
                expected_level.is_none(),
            )?;
        }
        let key_width = self.expectation.field_widths.key_width_in_bytes();
        if header.key_width != key_width {
            return Err(RecoveryFailure::InvariantViolated {
                invariant: "E142 走读同款",
                detail: "码 2 节点自述 key 宽与树的种类不符",
            });
        }
        if let Some(expected) = expected_level {
            if header.level != expected {
                return Err(violated("孩子的层级不是父层级减一"));
            }
        }
        if header.entries.is_empty() {
            return Err(violated("多层码 2 树的节点一条条目都没有"));
        }
        if header.level == 0 {
            return self.leaf_read_from_disk(pointer, bytes, header);
        }
        let internal_width = internal_entry_width_in_bytes(key_width);
        if header.entry_width < internal_width {
            return Err(RecoveryFailure::EntryNarrowerThanItsFieldTable {
                what: self.expectation.internal_entry_name,
                entry_bytes: header.entry_width,
                field_table_bytes: internal_width,
            });
        }
        let mut children: Vec<(CodeTwoTreeKey, NodeReadFromDisk)> =
            Vec::with_capacity(header.entries.len());
        // 迭代上界是这个节点的条目数；跨轮携带的是已经读回来的孩子（下一个孩子的分隔 key 要比上一个孩子头里的最大 key 大）。
        for entry in &header.entries {
            let (separator_bytes, child_pointer) =
                parse_internal_entry(entry, key_width).expect("条目宽上面判过不窄于 key 宽 + 86");
            let separator_key = self.key(&separator_bytes);
            if self.judges_every_header() {
                if let Some((previous_separator, previous_child)) = children.last() {
                    if separator_key <= *previous_separator {
                        return Err(violated("内部节点的分隔 key 不严格递增"));
                    }
                    if separator_key <= self.key(&previous_child.header.largest_key) {
                        return Err(violated("分隔 key 不大于左邻孩子头里的最大 key"));
                    }
                }
            }
            let child = self.read_node_and_its_subtree(
                &child_pointer,
                Some(header.level - 1),
                read_node,
                units_seen,
            )?;
            if self.judges_every_header() && separator_key > self.key(&child.header.smallest_key) {
                return Err(violated("分隔 key 大于孩子头里的最小 key"));
            }
            children.push((separator_key, child));
        }
        if self.judges_every_header() {
            let (_, first_child) = children.first().expect("上面判过节点不空");
            let (_, last_child) = children.last().expect("上面判过节点不空");
            if first_child.header.smallest_key != header.smallest_key
                || last_child.header.largest_key != header.largest_key
            {
                return Err(violated(
                    "内部节点的 key 区间不是子树覆盖区间（第一个孩子的最小 key 到最后一个孩子的最大 key）",
                ));
            }
        }
        Ok(NodeReadFromDisk {
            pointer: *pointer,
            bytes,
            header,
            children,
        })
    }

    fn leaf_read_from_disk(
        &self,
        pointer: &NodePointer,
        bytes: Vec<u8>,
        header: IndexNodeHeader,
    ) -> Result<NodeReadFromDisk, RecoveryFailure> {
        // 叶条目窄于它那棵树的字段表（记账 34、映射 55）不在这里判：这一步只切 key（`parse_index_node` 判过条目宽 ≥ key 宽），
        // 解条目的一方各按各的报（冷走读与重建报 `EntryNarrowerThanItsFieldTable`，挂载态报 `RecordMalformed`），
        // 与树长成多层之前逐字相同。
        if self.judges_every_header() {
            let key_width = self.expectation.field_widths.key_width_in_bytes();
            let keys_ascend = header
                .entries
                .windows(2)
                .all(|pair| self.key(&pair[0][..key_width]) < self.key(&pair[1][..key_width]));
            if !keys_ascend {
                return Err(violated("叶里的条目不按 key 严格递增"));
            }
        }
        Ok(NodeReadFromDisk {
            pointer: *pointer,
            bytes,
            header,
            children: Vec::new(),
        })
    }
}

fn flatten_read(
    node: NodeReadFromDisk,
    field_widths: CodeTwoKeyFieldWidths,
    nodes_by_level: &mut Vec<Vec<(CodeTwoTreeNodeContents, NodePointer, Vec<u8>)>>,
    leaf_entries_in_key_order: &mut Vec<Vec<u8>>,
) -> CodeTwoTreeNodePosition {
    let level = node.header.level;
    let key_width = field_widths.key_width_in_bytes();
    let contents = if level == 0 {
        let keys = node
            .header
            .entries
            .iter()
            .map(|entry| CodeTwoTreeKey::new(&entry[..key_width], field_widths))
            .collect();
        leaf_entries_in_key_order.extend(node.header.entries.iter().cloned());
        CodeTwoTreeNodeContents::Leaf { keys }
    } else {
        let children = node
            .children
            .into_iter()
            .map(|(separator_key, child)| CodeTwoTreeChild {
                separator_key,
                child: flatten_read(
                    child,
                    field_widths,
                    nodes_by_level,
                    leaf_entries_in_key_order,
                ),
            })
            .collect();
        CodeTwoTreeNodeContents::Internal { children }
    };
    let level_index = usize::from(level);
    if nodes_by_level.len() <= level_index {
        nodes_by_level.resize_with(level_index + 1, Vec::new);
    }
    let index_in_level =
        u32::try_from(nodes_by_level[level_index].len()).expect("一层的节点数装得进 u32");
    nodes_by_level[level_index].push((contents, node.pointer, node.bytes));
    CodeTwoTreeNodePosition {
        level,
        index_in_level,
    }
}

/// 在一版树里按 key 找它所在的那片叶：从根按分隔 key 往下走（最后一个分隔 key ≤ key 的孩子；比第一个还小就走第 0 个）。
/// 交回叶的位置；树还没建过交回 `None`。
#[must_use]
pub fn leaf_position_routed_to(
    shape: &CodeTwoTreeShape,
    key: &CodeTwoTreeKey,
) -> Option<CodeTwoTreeNodePosition> {
    let mut position = shape.root()?.position;
    // 迭代上界：每一轮下降一层。
    loop {
        match &shape.node(position)?.contents {
            CodeTwoTreeNodeContents::Leaf { .. } => return Some(position),
            CodeTwoTreeNodeContents::Internal { children } => {
                let index = children
                    .iter()
                    .rposition(|child| child.separator_key <= *key)
                    .unwrap_or(0);
                position = children.get(index)?.child;
            }
        }
    }
}

/// 一版树里每个节点头里该写的 key 区间（子树覆盖区间），按 `shape.nodes()` 同序。
///
/// # Panics
/// 形状里有空节点：规划与读回都不交出空节点（空的叶只在树删到空时出现，调用方的两棵树不会删到空）。
#[must_use]
pub fn key_ranges_in_bump_order(shape: &CodeTwoTreeShape) -> Vec<(Vec<u8>, Vec<u8>)> {
    shape
        .nodes()
        .iter()
        .map(|node| {
            let (smallest, largest) = shape
                .key_range_of_the_subtree(node.position)
                .expect("规划与读回都不交出空节点");
            (smallest.bytes().to_vec(), largest.bytes().to_vec())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_FIELD_WIDTHS: CodeTwoKeyFieldWidths = CodeTwoKeyFieldWidths(&[8]);

    fn key(value: u64) -> CodeTwoTreeKey {
        CodeTwoTreeKey::new(&value.to_le_bytes(), TEST_FIELD_WIDTHS)
    }

    fn keys(values: impl IntoIterator<Item = u64>) -> BTreeSet<CodeTwoTreeKey> {
        values.into_iter().map(key).collect()
    }

    fn capacity(leaf_entries: usize, internal_entries: usize) -> CodeTwoTreeNodeCapacity {
        CodeTwoTreeNodeCapacity {
            leaf_entries,
            internal_entries,
        }
    }

    /// 每片叶装的 key，按层级 0 从左到右。
    fn leaves(shape: &CodeTwoTreeShape) -> Vec<Vec<u64>> {
        shape
            .nodes()
            .iter()
            .filter_map(|node| match &node.contents {
                CodeTwoTreeNodeContents::Leaf { keys } => Some(
                    keys.iter()
                        .map(|leaf_key| u64::from_le_bytes(leaf_key.bytes().try_into().expect("8")))
                        .collect(),
                ),
                CodeTwoTreeNodeContents::Internal { .. } => None,
            })
            .collect()
    }

    #[test]
    fn internal_entries_are_the_key_plus_an_eighty_six_byte_child_pointer() {
        use singlefs_format::{
            ACCOUNTING_INTERNAL_ENTRY_BYTES, ACCOUNTING_KEY_BYTES,
            CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES, MAPPING_KEY_BYTES,
        };
        let width = |key_bytes: u64| {
            u64::try_from(internal_entry_width_in_bytes(
                usize::try_from(key_bytes).expect("key 宽"),
            ))
            .expect("条目宽")
        };
        assert_eq!(width(ACCOUNTING_KEY_BYTES), ACCOUNTING_INTERNAL_ENTRY_BYTES);
        assert_eq!(
            width(MAPPING_KEY_BYTES),
            CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES
        );
        assert_eq!(
            CodeTwoKeyFieldWidths::ACCOUNTING.key_width_in_bytes(),
            usize::try_from(ACCOUNTING_KEY_BYTES).expect("22")
        );
        assert_eq!(
            CodeTwoKeyFieldWidths::CENTRAL_MAPPING.key_width_in_bytes(),
            usize::try_from(MAPPING_KEY_BYTES).expect("27")
        );
    }

    #[test]
    fn inserting_nine_keys_into_a_leaf_of_eight_splits_it_in_the_middle_and_grows_a_root() {
        let plan = plan_the_tree_after_this_publish(
            &CodeTwoTreeShape::default(),
            &keys(1..=9),
            capacity(8, 4),
        )
        .expect("两层装得下");
        assert_eq!(plan.shape.height(), 2);
        assert_eq!(
            leaves(&plan.shape),
            vec![vec![1, 2, 3, 4, 5], vec![6, 7, 8, 9]]
        );
        assert!(plan
            .origins
            .iter()
            .all(|origin| *origin == CodeTwoTreeNodeOrigin::RewrittenThisPublish));
        let CodeTwoTreeNodeContents::Internal { children } =
            &plan.shape.root().expect("有根").contents
        else {
            panic!("根是内部节点");
        };
        assert_eq!(children[0].separator_key, key(1));
        assert_eq!(
            children[1].separator_key,
            key(6),
            "右半的分隔 key 取它自己的最小 key"
        );
    }

    #[test]
    fn a_key_below_the_leftmost_separator_lowers_it_and_leaves_the_right_leaf_carried() {
        let before = plan_the_tree_after_this_publish(
            &CodeTwoTreeShape::default(),
            &keys([10, 20, 30, 40, 50]),
            capacity(4, 4),
        )
        .expect("两层")
        .shape;
        assert_eq!(leaves(&before), vec![vec![10, 20, 30], vec![40, 50]]);
        let after = plan_the_tree_after_this_publish(
            &before,
            &keys([5, 10, 20, 30, 40, 50]),
            capacity(4, 4),
        )
        .expect("两层");
        let CodeTwoTreeNodeContents::Internal { children } =
            &after.shape.root().expect("根").contents
        else {
            panic!("根是内部节点");
        };
        assert_eq!(
            children[0].separator_key,
            key(5),
            "最左分隔 key 压低到新插的 key"
        );
        assert_eq!(
            after.origin_of(CodeTwoTreeNodePosition {
                level: 0,
                index_in_level: 1
            }),
            CodeTwoTreeNodeOrigin::CarriedFrom(CodeTwoTreeNodePosition {
                level: 0,
                index_in_level: 1
            }),
            "右叶一个 key 都没动：照抄"
        );
        assert_eq!(
            after.replaced_previous_nodes,
            vec![
                CodeTwoTreeNodePosition {
                    level: 0,
                    index_in_level: 0
                },
                CodeTwoTreeNodePosition {
                    level: 1,
                    index_in_level: 0
                }
            ],
            "改了的左叶与根被换下"
        );
    }

    #[test]
    fn deleting_every_key_of_a_leaf_drops_it_and_a_root_left_with_one_child_hands_the_root_to_it() {
        let before = plan_the_tree_after_this_publish(
            &CodeTwoTreeShape::default(),
            &keys([10, 20, 30, 40, 50]),
            capacity(4, 4),
        )
        .expect("两层")
        .shape;
        let after = plan_the_tree_after_this_publish(&before, &keys([10, 20, 30]), capacity(4, 4))
            .expect("删成一层");
        assert_eq!(after.shape.height(), 1, "根只剩一个孩子：降高一层");
        assert_eq!(leaves(&after.shape), vec![vec![10, 20, 30]]);
        assert_eq!(
            after.origins,
            vec![CodeTwoTreeNodeOrigin::CarriedFrom(
                CodeTwoTreeNodePosition {
                    level: 0,
                    index_in_level: 0
                }
            )],
            "当根的孩子不重写"
        );
    }

    #[test]
    fn a_leaf_split_that_overflows_a_full_root_splits_the_root_too_and_the_tree_grows_to_three_levels(
    ) {
        let before = plan_the_tree_after_this_publish(
            &CodeTwoTreeShape::default(),
            &keys(1..=4),
            capacity(2, 2),
        )
        .expect("两层");
        assert_eq!(before.shape.height(), 2);
        assert_eq!(leaves(&before.shape), vec![vec![1, 2], vec![3, 4]]);
        let after = plan_the_tree_after_this_publish(&before.shape, &keys(1..=5), capacity(2, 2))
            .expect("三层");
        assert_eq!(after.shape.height(), 3, "叶切开让根装不下，根也切开");
        assert_eq!(leaves(&after.shape), vec![vec![1, 2], vec![3, 4], vec![5]]);
    }

    #[test]
    fn the_level_byte_bounds_the_height() {
        assert_eq!(u8::MAX.checked_add(1), None);
        let mut root = PlanningNode::Internal {
            level: u8::MAX,
            children: vec![
                PlanningChild {
                    separator_key: key(1),
                    node: PlanningNode::empty_leaf(),
                },
                PlanningChild {
                    separator_key: key(2),
                    node: PlanningNode::empty_leaf(),
                },
            ],
            carried_from: None,
        };
        // 根的层级是 255、孩子是空叶（只为造出这一格，层级不连贯）：插一把 key 进孩子不分裂，根不切；
        // 把内部容量压到 1 让根装不下，根分裂要长出第 256 层 ⇒ 拒绝。
        let refused = insert_at_the_root(&mut root, key(3), capacity(4, 1));
        assert_eq!(refused, Err(CodeTwoTreeRefusal::HeightBeyondTheLevelField));
    }
}
