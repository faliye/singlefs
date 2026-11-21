//! 节点与节点集、释放链的闭包与深度、旗线与重现门、取整中位数与层数。

use super::*;

// ---- 节点与几何（第一节读法 3；独立于实现的位置算术）----

/// 分配记录树里的一个节点：位置三元组 `(层级, 盘号, 盘内序号)` 或根。派生的全序把根排在最末（同实现）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum Node {
    Position { level: u32, device: u32, index_in_device: u64 },
    Root,
}

pub(super) type NodeSet = BTreeSet<Node>;
/// 上一版每个节点的两份旧落点：节点 → [(盘号, 槽号)]。
pub(super) type PointerGraph = BTreeMap<Node, Vec<(u32, u64)>>;

/// 合成日志里用的抽象节点：编号写成第 0 层第 0 盘的盘内序号。
pub(super) fn synthetic_node(number: u64) -> Node {
    Node::Position { level: 0, device: 0, index_in_device: number }
}

pub(super) fn synthetic_set(numbers: &[u64]) -> NodeSet {
    numbers.iter().map(|number| synthetic_node(*number)).collect()
}

pub(super) fn parse_node(text: &str) -> Option<Node> {
    if text == "root" {
        return Some(Node::Root);
    }
    let mut parts = text.split('/');
    let node = Node::Position {
        level: parts.next()?.parse().ok()?,
        device: parts.next()?.parse().ok()?,
        index_in_device: parts.next()?.parse().ok()?,
    };
    parts.next().is_none().then_some(node)
}

/// `-` 是空；坏一个节点整串读不出（交 `None`，不当空）。
pub(super) fn parse_node_set(text: &str) -> Option<NodeSet> {
    if text == "-" {
        return Some(NodeSet::new());
    }
    text.split(',').map(parse_node).collect()
}

/// 一个槽所在的那片叶（`allocation_record_tree.rs` 的 `leaf_of_slot`：槽号 ÷ 812）。
pub(super) fn leaf_of_slot(device: u32, slot: u64) -> Node {
    Node::Position { level: 0, device, index_in_device: slot / LEAF_SLOTS }
}

/// 一个节点的父节点（`parent_of`：下标 ÷ 169；父的层级等于根的层级时父是根）；根没有父。
pub(super) fn parent_of(node: Node, root_level: u32) -> Option<Node> {
    match node {
        Node::Root => None,
        Node::Position { level, device, index_in_device } => {
            let parent_level = level + 1;
            if parent_level == root_level {
                Some(Node::Root)
            } else {
                Some(Node::Position { level: parent_level, device, index_in_device: index_in_device / INTERNAL_FANOUT })
            }
        }
    }
}

/// 释放链闭包（独立模型）：从种子起，每个节点按上一版指针查它的两份旧落点，旧落点所在的叶与叶的全部祖先并进来，新并进的接着走。
/// 上一版没有的节点没有旧落点，走到它为止。
pub(super) fn release_chain_closure(graph: &PointerGraph, root_level: u32, seeds: &NodeSet) -> NodeSet {
    let mut reached = seeds.clone();
    let mut waiting: Vec<Node> = seeds.iter().copied().collect();
    while let Some(node) = waiting.pop() {
        let Some(locations) = graph.get(&node) else {
            continue;
        };
        for (device, slot) in locations {
            let mut climbing = Some(leaf_of_slot(*device, *slot));
            while let Some(ancestor) = climbing {
                if reached.insert(ancestor) {
                    waiting.push(ancestor);
                }
                climbing = parent_of(ancestor, root_level);
            }
        }
    }
    reached
}

/// 单个节点作种子时释放链走的层数（一层 = 待走的这一批全部走完一遍；至少 1）。
pub(super) fn release_chain_depth(graph: &PointerGraph, root_level: u32, start: Node) -> u64 {
    let mut reached: NodeSet = BTreeSet::from([start]);
    let mut frontier = vec![start];
    let mut depth = 0;
    while !frontier.is_empty() {
        depth += 1;
        let mut next = Vec::new();
        for node in frontier {
            let Some(locations) = graph.get(&node) else {
                continue;
            };
            for (device, slot) in locations {
                let mut climbing = Some(leaf_of_slot(*device, *slot));
                while let Some(ancestor) = climbing {
                    if reached.insert(ancestor) {
                        next.push(ancestor);
                    }
                    climbing = parent_of(ancestor, root_level);
                }
            }
        }
        frontier = next;
    }
    depth
}

// ---- 旗（读法 1）----

/// 下中位数：排好序取第 ⌊(n−1)/2⌋ 个；空取 0。
pub(super) fn lower_median(values: &[u64]) -> u64 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted.get(sorted.len().saturating_sub(1) / 2).copied().unwrap_or(0)
}

/// 旗线：`max(下限, 4 × 下中位数)`；一个项一次挂载里全部操作的 `final_nodes` 一起算。
pub(super) fn spike_line(floor: u64, final_nodes: &[u64]) -> u64 {
    floor.max(SPIKE_MULTIPLE * lower_median(final_nodes))
}

/// 一次挂载里每个操作在三条旗线下是不是旗上，次序同 `FLOOR_LINES`。
pub(super) fn flags_by_line(final_nodes: &[u64]) -> Vec<[bool; 3]> {
    let lines: Vec<u64> = FLOOR_LINES.iter().map(|floor| spike_line(*floor, final_nodes)).collect();
    final_nodes.iter().map(|nodes| [*nodes >= lines[0], *nodes >= lines[1], *nodes >= lines[2]]).collect()
}

/// Q0 复现闸：一个点的 `walk_forward` 里旗上操作的个数（三条旗线）与判定。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReproductionGate {
    pub(super) flagged_by_line: [usize; 3],
    pub(super) verdict: &'static str,
}

pub(super) fn reproduction_gate(final_nodes: &[u64]) -> ReproductionGate {
    let flags = flags_by_line(final_nodes);
    let count = |position: usize| flags.iter().filter(|row| row[position]).count();
    let flagged_by_line = [count(0), count(1), count(2)];
    let verdict = if flagged_by_line[MAIN_FLOOR_LINE_POSITION] > 0 {
        "reproduced"
    } else if flagged_by_line[LOW_FLOOR_LINE_POSITION] > 0 {
        "only_at_the_low_line"
    } else {
        "not_reproduced"
    };
    ReproductionGate { flagged_by_line, verdict }
}

/// 一个项一次挂载里旗上操作超过半数：中位数被拉高，旗线失去意义。
pub(super) fn median_is_inflated(flags: &[bool]) -> bool {
    flags.iter().filter(|flagged| **flagged).count() * 2 > flags.len()
}
/// inode 叶容器每片记录数（E203 模型同一个常量）。
pub(super) const INODE_LEAF_RECORDS: u64 = 233;
/// 每段槽数（E203 模型同一个常量）。
pub(super) const SEGMENT_SLOTS: u64 = 64;

pub(super) fn lg(value: u64) -> u64 {
    u64::from(u64::BITS - value.leading_zeros())
}

/// 每层 64 叉的分层位图要几层：最小的 l ≥ 1 使 64ˡ ≥ n（E203 同一个式子）。
pub(super) fn levels_64(value: u64) -> u64 {
    let mut level = 1_u32;
    while 64_u64.pow(level) < value {
        level += 1;
    }
    u64::from(level)
}
