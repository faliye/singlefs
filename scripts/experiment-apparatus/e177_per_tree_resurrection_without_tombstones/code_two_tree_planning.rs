//! 第二种写法的节点位置、内容与形状、规划用的节点与逐层插入删除、拍平与整棵树的规划。

use super::*;

// ---------------------------------------------------------------------------------------------
// 多层码 2 树（记账树、中央映射树）：照 `crates/singlefs-core/src/code_two_tree.rs:425-705` 的规则手写
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct CodeTwoNodePosition {
    level: u8,
    index_in_level: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CodeTwoNodeContents {
    Leaf { keys: Vec<KeyFields> },
    Internal { children: Vec<(KeyFields, CodeTwoNodePosition)> },
}

/// 一版树的形状：节点按位置的全序（层级升序、同层按 key 升序），根在最末。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CodeTwoShape {
    pub(super) nodes: Vec<(CodeTwoNodePosition, CodeTwoNodeContents)>,
}

impl CodeTwoShape {
    fn root_position(&self) -> Option<CodeTwoNodePosition> {
        self.nodes.last().map(|(position, _)| *position)
    }

    pub(super) fn contents_at(&self, position: CodeTwoNodePosition) -> &CodeTwoNodeContents {
        let found = self
            .nodes
            .binary_search_by_key(&position, |(node_position, _)| *node_position)
            .expect("形状里的子引用指着形状里的节点");
        &self.nodes[found].1
    }

    pub(super) fn keys_in_order(&self) -> Vec<KeyFields> {
        self.nodes
            .iter()
            .filter_map(|(_, contents)| match contents {
                CodeTwoNodeContents::Leaf { keys } => Some(keys.iter().copied()),
                CodeTwoNodeContents::Internal { .. } => None,
            })
            .flatten()
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CodeTwoOrigin {
    CarriedFrom(CodeTwoNodePosition),
    Rewritten,
}

#[derive(Clone, Debug)]
pub(super) struct CodeTwoPlan {
    pub(super) shape: CodeTwoShape,
    /// 与 `shape.nodes` 同序。
    pub(super) origins: Vec<CodeTwoOrigin>,
    /// 上一版里这次被换下的节点。
    pub(super) replaced_previous: Vec<CodeTwoNodePosition>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CodeTwoCapacity {
    leaf_entries: usize,
    internal_entries: usize,
}

/// 记账树：叶 477 条、内部 150 条（执行员修订 2 核对 4 M10）。
pub(super) fn accounting_tree_capacity() -> CodeTwoCapacity {
    CodeTwoCapacity {
        leaf_entries: u64_as_index(index_node_entry_capacity(ACCOUNTING_KEY_BYTES, ACCOUNTING_ENTRY_BYTES)),
        internal_entries: u64_as_index(index_node_entry_capacity(
            ACCOUNTING_KEY_BYTES,
            ACCOUNTING_KEY_BYTES + NODE_POINTER_BYTES,
        )),
    }
}

/// 中央映射树：叶 294 条、内部 143 条（M10）。
pub(super) fn mapping_tree_capacity() -> CodeTwoCapacity {
    CodeTwoCapacity {
        leaf_entries: u64_as_index(index_node_entry_capacity(MAPPING_KEY_BYTES, MAPPING_ENTRY_BYTES)),
        internal_entries: u64_as_index(index_node_entry_capacity(
            MAPPING_KEY_BYTES,
            MAPPING_KEY_BYTES + NODE_POINTER_BYTES,
        )),
    }
}

#[derive(Clone, Debug)]
enum PlanningNode {
    Leaf {
        keys: Vec<KeyFields>,
        carried_from: Option<CodeTwoNodePosition>,
    },
    Internal {
        level: u8,
        children: Vec<PlanningChild>,
        carried_from: Option<CodeTwoNodePosition>,
    },
}

#[derive(Clone, Debug)]
struct PlanningChild {
    separator_key: KeyFields,
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

    fn mark_rewritten(&mut self) {
        match self {
            PlanningNode::Leaf { carried_from, .. } | PlanningNode::Internal { carried_from, .. } => {
                *carried_from = None;
            }
        }
    }

    fn smallest_key(&self) -> Option<KeyFields> {
        match self {
            PlanningNode::Leaf { keys, .. } => keys.first().copied(),
            PlanningNode::Internal { children, .. } => children.first()?.node.smallest_key(),
        }
    }
}

fn planning_node_of(shape: &CodeTwoShape, position: CodeTwoNodePosition) -> PlanningNode {
    match shape.contents_at(position) {
        CodeTwoNodeContents::Leaf { keys } => PlanningNode::Leaf {
            keys: keys.clone(),
            carried_from: Some(position),
        },
        CodeTwoNodeContents::Internal { children } => PlanningNode::Internal {
            level: position.level,
            children: children
                .iter()
                .map(|(separator_key, child)| PlanningChild {
                    separator_key: *separator_key,
                    node: planning_node_of(shape, *child),
                })
                .collect(),
            carried_from: Some(position),
        },
    }
}

/// 插入找孩子：最后一个分隔 key ≤ key 的那一个；比第一个还小就落到第 0 个并把它的分隔 key 压低（规则 ④）。
fn route_for_insertion(children: &mut [PlanningChild], key: &KeyFields) -> usize {
    match children.iter().rposition(|child| child.separator_key <= *key) {
        Some(found) => found,
        None => {
            children[0].separator_key = *key;
            0
        }
    }
}

fn route_for_deletion(children: &[PlanningChild], key: &KeyFields) -> usize {
    children
        .iter()
        .rposition(|child| child.separator_key <= *key)
        .unwrap_or(0)
}

/// 从中间切：左半留 ⌈n ÷ 2⌉ 条，右半的分隔 key 取它自己的最小 key（规则 ②）。
fn split_in_the_middle(node: &mut PlanningNode) -> PlanningChild {
    match node {
        PlanningNode::Leaf { keys, .. } => {
            let right_keys = keys.split_off(keys.len().div_ceil(2));
            PlanningChild {
                separator_key: right_keys[0],
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
                separator_key: right_children[0].separator_key,
                node: PlanningNode::Internal {
                    level: *level,
                    children: right_children,
                    carried_from: None,
                },
            }
        }
    }
}

fn insert_below(node: &mut PlanningNode, key: KeyFields, capacity: CodeTwoCapacity) -> Option<PlanningChild> {
    node.mark_rewritten();
    let overflows = match node {
        PlanningNode::Leaf { keys, .. } => {
            let insertion_point = keys.partition_point(|existing| *existing < key);
            assert!(keys.get(insertion_point) != Some(&key), "插入的 key 不在树里");
            keys.insert(insertion_point, key);
            keys.len() > capacity.leaf_entries
        }
        PlanningNode::Internal { children, .. } => {
            let child_position = route_for_insertion(children, &key);
            if let Some(right_half) = insert_below(&mut children[child_position].node, key, capacity) {
                children.insert(child_position + 1, right_half);
            }
            children.len() > capacity.internal_entries
        }
    };
    if overflows {
        Some(split_in_the_middle(node))
    } else {
        None
    }
}

fn insert_at_the_root(root: &mut PlanningNode, key: KeyFields, capacity: CodeTwoCapacity) {
    let Some(right_half) = insert_below(root, key, capacity) else {
        return;
    };
    let new_root_level = root.level() + 1;
    let left_half = std::mem::replace(root, PlanningNode::empty_leaf());
    let left_separator = left_half.smallest_key().expect("刚切开的节点左半不空");
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
}

/// 删一把 key；删空的孩子从父节点里摘掉（规则 ③）。交回找没找到。
fn delete_below(node: &mut PlanningNode, key: &KeyFields) -> bool {
    let found = match node {
        PlanningNode::Leaf { keys, .. } => match keys.binary_search(key) {
            Ok(position) => {
                keys.remove(position);
                true
            }
            Err(_) => false,
        },
        PlanningNode::Internal { children, .. } => {
            let child_position = route_for_deletion(children, key);
            if !delete_below(&mut children[child_position].node, key) {
                return false;
            }
            if children[child_position].node.entry_count() == 0 {
                children.remove(child_position);
            }
            true
        }
    };
    if found {
        node.mark_rewritten();
    }
    found
}

fn delete_at_the_root(root: &mut PlanningNode, key: &KeyFields) {
    assert!(delete_below(root, key), "上一版的形状按分隔 key 走得到它自己叶里的 key");
    loop {
        let only_child = match root {
            PlanningNode::Internal { children, .. } if children.is_empty() => {
                *root = PlanningNode::empty_leaf();
                return;
            }
            PlanningNode::Internal { children, .. } if children.len() == 1 => {
                children.pop().expect("刚判过恰一个孩子").node
            }
            PlanningNode::Internal { .. } | PlanningNode::Leaf { .. } => return,
        };
        *root = only_child;
    }
}

type NodesByLevel = Vec<Vec<(CodeTwoNodeContents, CodeTwoOrigin)>>;

fn flatten(node: PlanningNode, nodes_by_level: &mut NodesByLevel) -> CodeTwoNodePosition {
    let level = node.level();
    let (contents, carried_from) = match node {
        PlanningNode::Leaf { keys, carried_from } => (CodeTwoNodeContents::Leaf { keys }, carried_from),
        PlanningNode::Internal {
            children,
            carried_from,
            ..
        } => {
            let mut flattened_children = Vec::new();
            for child in children {
                let child_position = flatten(child.node, nodes_by_level);
                flattened_children.push((child.separator_key, child_position));
            }
            (
                CodeTwoNodeContents::Internal {
                    children: flattened_children,
                },
                carried_from,
            )
        }
    };
    let level_position = usize::from(level);
    if nodes_by_level.len() <= level_position {
        nodes_by_level.resize_with(level_position + 1, Vec::new);
    }
    let index_in_level = u32::try_from(nodes_by_level[level_position].len()).expect("一层的节点数装得进 u32");
    let origin = match carried_from {
        Some(previous_position) => CodeTwoOrigin::CarriedFrom(previous_position),
        None => CodeTwoOrigin::Rewritten,
    };
    nodes_by_level[level_position].push((contents, origin));
    CodeTwoNodePosition {
        level,
        index_in_level,
    }
}

/// 这次之后树长什么样：先按 key 升序删上一版有、这一版没有的，再按 key 升序插这一版有、上一版没有的。
pub(super) fn plan_code_two_tree(
    previous: &CodeTwoShape,
    keys_after_this_publish: &BTreeSet<KeyFields>,
    capacity: CodeTwoCapacity,
) -> CodeTwoPlan {
    assert!(!keys_after_this_publish.is_empty(), "记账树、映射树这一版恒有 key");
    let previous_keys: BTreeSet<KeyFields> = previous.keys_in_order().into_iter().collect();
    let mut root = match previous.root_position() {
        Some(root_position) => planning_node_of(previous, root_position),
        None => PlanningNode::empty_leaf(),
    };
    for deleted in previous_keys.difference(keys_after_this_publish) {
        delete_at_the_root(&mut root, deleted);
    }
    for inserted in keys_after_this_publish.difference(&previous_keys) {
        insert_at_the_root(&mut root, *inserted, capacity);
    }
    let mut nodes_by_level: NodesByLevel = Vec::new();
    flatten(root, &mut nodes_by_level);
    let mut nodes = Vec::new();
    let mut origins = Vec::new();
    for (level, nodes_of_this_level) in nodes_by_level.into_iter().enumerate() {
        for (index_in_level, (contents, origin)) in nodes_of_this_level.into_iter().enumerate() {
            nodes.push((
                CodeTwoNodePosition {
                    level: u8::try_from(level).expect("层级是 1 字节"),
                    index_in_level: u32::try_from(index_in_level).expect("一层的节点数装得进 u32"),
                },
                contents,
            ));
            origins.push(origin);
        }
    }
    let carried: BTreeSet<CodeTwoNodePosition> = origins
        .iter()
        .filter_map(|origin| match origin {
            CodeTwoOrigin::CarriedFrom(previous_position) => Some(*previous_position),
            CodeTwoOrigin::Rewritten => None,
        })
        .collect();
    let replaced_previous = previous
        .nodes
        .iter()
        .map(|(position, _)| *position)
        .filter(|position| !carried.contains(position))
        .collect();
    CodeTwoPlan {
        shape: CodeTwoShape { nodes },
        origins,
        replaced_previous,
    }
}

/// 一版码 2 树：形状加每个节点这一版的单元（与 `shape.nodes` 同序）。
#[derive(Clone, Debug, Default)]
pub(super) struct CodeTwoTreeVersion {
    pub(super) shape: CodeTwoShape,
    pub(super) units: Vec<UnitIdentifier>,
}

impl CodeTwoTreeVersion {
    pub(super) fn unit_at(&self, position: CodeTwoNodePosition) -> UnitIdentifier {
        let found = self
            .shape
            .nodes
            .binary_search_by_key(&position, |(node_position, _)| *node_position)
            .expect("上一版的位置在上一版的形状里");
        self.units[found]
    }
}

#[cfg(test)]
#[path = "code_two_tree_planning_tests.rs"]
mod tests;
