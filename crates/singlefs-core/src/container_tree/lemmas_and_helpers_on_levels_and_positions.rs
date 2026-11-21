//! 容器树形状的各层（每层各节点的孩子数）与节点位置上的引理和执行侧辅助函数：孩子数之和、覆盖节点与祖先、在最右端开一片、
//! 重写的容器在最右端、位置落在形状里；照抄各层、判每个节点至少一个孩子、上一层计数下一层节点数、位置在不在单子里的执行函数。

use super::*;

verus! {

// ===== 形状的引理 =====

/// 前 `count` 个之和加上第 `count` 个就是前 `count + 1` 个之和。
pub(super) proof fn lemma_children_count_sum_of_one_more(children_of_each_node: Seq<usize>, count: int)
    requires
        0 <= count < children_of_each_node.len(),
    ensures
        children_count_sum(children_of_each_node.take(count + 1)) == children_count_sum(children_of_each_node.take(count))
            + children_of_each_node[count],
{
    assert(children_of_each_node.take(count + 1).drop_last() =~= children_of_each_node.take(count));
}

/// 取全部就是整层。
proof fn lemma_children_count_sum_of_the_whole_level(children_of_each_node: Seq<usize>)
    ensures
        children_count_sum(children_of_each_node.take(children_of_each_node.len() as int)) == children_count_sum(
            children_of_each_node,
        ),
{
    assert(children_of_each_node.take(children_of_each_node.len() as int) =~= children_of_each_node);
}

/// 孩子数之和不小于 0。
pub(super) proof fn lemma_children_count_sum_is_not_negative(children_of_each_node: Seq<usize>)
    ensures
        children_count_sum(children_of_each_node) >= 0,
    decreases children_of_each_node.len(),
{
    if children_of_each_node.len() > 0 {
        lemma_children_count_sum_is_not_negative(children_of_each_node.drop_last());
    }
}

/// 前缀之和随前缀变长不减，不超过整层之和。
pub(super) proof fn lemma_children_count_prefix_sum_is_monotone(children_of_each_node: Seq<usize>, shorter: int, longer: int)
    requires
        0 <= shorter <= longer <= children_of_each_node.len(),
    ensures
        children_count_sum(children_of_each_node.take(shorter)) <= children_count_sum(children_of_each_node.take(longer)),
        children_count_sum(children_of_each_node.take(longer)) <= children_count_sum(children_of_each_node),
    decreases children_of_each_node.len() - shorter,
{
    if shorter < children_of_each_node.len() {
        lemma_children_count_sum_of_one_more(children_of_each_node, shorter);
        if shorter < longer {
            lemma_children_count_prefix_sum_is_monotone(children_of_each_node, shorter + 1, longer);
        } else {
            lemma_children_count_prefix_sum_is_monotone(children_of_each_node, shorter + 1, shorter + 1);
        }
    } else {
        lemma_children_count_sum_of_the_whole_level(children_of_each_node);
    }
}

/// 每个节点至少一个孩子时，孩子数之和不小于节点数。
proof fn lemma_children_count_sum_is_at_least_the_node_count(children_of_each_node: Seq<usize>)
    requires
        forall|node: int| 0 <= node < children_of_each_node.len() ==> #[trigger] children_of_each_node[node] >= 1,
    ensures
        children_count_sum(children_of_each_node) >= children_of_each_node.len(),
    decreases children_of_each_node.len(),
{
    if children_of_each_node.len() > 0 {
        let shorter = children_of_each_node.drop_last();
        assert forall|node: int| 0 <= node < shorter.len() implies #[trigger] shorter[node] >= 1 by {
            assert(shorter[node] == children_of_each_node[node]);
        }
        lemma_children_count_sum_is_at_least_the_node_count(shorter);
        assert(children_of_each_node.last() >= 1);
    }
}

/// 末尾加一个，和加上它。
proof fn lemma_children_count_sum_after_a_push(children_of_each_node: Seq<usize>, value: usize)
    ensures
        children_count_sum(children_of_each_node.push(value)) == children_count_sum(children_of_each_node) + value,
{
    assert(children_of_each_node.push(value).drop_last() =~= children_of_each_node);
}

/// 改最右那一个，和随之变。
proof fn lemma_children_count_sum_after_an_update_of_the_last(children_of_each_node: Seq<usize>, value: usize)
    requires
        children_of_each_node.len() >= 1,
    ensures
        children_count_sum(children_of_each_node.update(children_of_each_node.len() - 1, value)) == children_count_sum(
            children_of_each_node,
        ) - children_of_each_node.last() + value,
{
    assert(children_of_each_node.update(children_of_each_node.len() - 1, value).drop_last()
        =~= children_of_each_node.drop_last());
}

/// 只有一个孩子数的一层，和就是它。
proof fn lemma_children_count_sum_of_a_single_node(value: usize)
    ensures
        children_count_sum(seq![value]) == value,
{
    reveal_with_fuel(children_count_sum, 2);
    assert(seq![value].drop_last() =~= Seq::<usize>::empty());
}

/// 形状良好时，每一层的节点数不超过容器数。
pub(super) proof fn lemma_every_level_has_at_most_as_many_nodes_as_containers(levels: Seq<Seq<usize>>, level_index: int)
    requires
        container_tree_shape_is_well_formed(levels),
        0 <= level_index < levels.len(),
    ensures
        levels[level_index].len() <= container_count(levels),
    decreases level_index,
{
    assert forall|node: int| 0 <= node < levels[level_index].len() implies #[trigger] levels[level_index][node] >= 1 by {
        assert(levels[level_index][node] >= 1);
    }
    lemma_children_count_sum_is_at_least_the_node_count(levels[level_index]);
    if level_index > 0 {
        assert(children_count_sum(levels[level_index]) == levels[level_index - 1].len());
        lemma_every_level_has_at_most_as_many_nodes_as_containers(levels, level_index - 1);
    }
}

/// 一层里最右那个节点罩住下一层的最后一个。
proof fn lemma_the_rightmost_node_holds_the_last_child(nodes_of_the_level: Seq<usize>)
    requires
        nodes_of_the_level.len() >= 1,
        forall|node: int| 0 <= node < nodes_of_the_level.len() ==> #[trigger] nodes_of_the_level[node] >= 1,
    ensures
        child_lies_under_node(
            nodes_of_the_level,
            nodes_of_the_level.len() - 1,
            children_count_sum(nodes_of_the_level) - 1,
        ),
{
    let last = nodes_of_the_level.len() - 1;
    lemma_children_count_sum_of_one_more(nodes_of_the_level, last);
    lemma_children_count_sum_of_the_whole_level(nodes_of_the_level);
    assert(nodes_of_the_level[last] >= 1);
}

/// 一层里一个孩子至多落在一个节点下（前缀孩子数之和随节点不减）。
proof fn lemma_a_child_lies_under_at_most_one_node(nodes_of_the_level: Seq<usize>, node: int, other_node: int, child: int)
    requires
        child_lies_under_node(nodes_of_the_level, node, child),
        child_lies_under_node(nodes_of_the_level, other_node, child),
    ensures
        node == other_node,
{
    if node < other_node {
        lemma_children_count_prefix_sum_is_monotone(nodes_of_the_level, node + 1, other_node);
    } else if other_node < node {
        lemma_children_count_prefix_sum_is_monotone(nodes_of_the_level, other_node + 1, node);
    }
}

/// 从第 `node` 个起往右找，找到的就是罩住它的那一个节点。
proof fn lemma_covering_node_from(nodes_of_the_level: Seq<usize>, child: int, node: int)
    requires
        0 <= node <= nodes_of_the_level.len(),
        children_count_sum(nodes_of_the_level.take(node)) <= child,
        child < children_count_sum(nodes_of_the_level),
    ensures
        node <= covering_node_from(nodes_of_the_level, child, node) < nodes_of_the_level.len(),
        child_lies_under_node(nodes_of_the_level, covering_node_from(nodes_of_the_level, child, node), child),
    decreases nodes_of_the_level.len() - node,
{
    if node >= nodes_of_the_level.len() {
        lemma_children_count_sum_of_the_whole_level(nodes_of_the_level);
    } else if child >= children_count_sum(nodes_of_the_level.take(node + 1)) {
        lemma_covering_node_from(nodes_of_the_level, child, node + 1);
    }
}

/// 落在这一层孩子数之和以内的孩子，罩住它的节点就是从头找到的那一个，别的节点都不罩它。
proof fn lemma_covering_node(nodes_of_the_level: Seq<usize>, child: int)
    requires
        0 <= child < children_count_sum(nodes_of_the_level),
    ensures
        0 <= covering_node_from(nodes_of_the_level, child, 0) < nodes_of_the_level.len(),
        child_lies_under_node(nodes_of_the_level, covering_node_from(nodes_of_the_level, child, 0), child),
        forall|node: int| #[trigger] child_lies_under_node(nodes_of_the_level, node, child) ==> node == covering_node_from(
            nodes_of_the_level,
            child,
            0,
        ),
{
    assert(nodes_of_the_level.take(0) =~= Seq::<usize>::empty());
    lemma_covering_node_from(nodes_of_the_level, child, 0);
    assert forall|node: int| #[trigger] child_lies_under_node(nodes_of_the_level, node, child) implies node
        == covering_node_from(nodes_of_the_level, child, 0) by {
        lemma_a_child_lies_under_at_most_one_node(
            nodes_of_the_level,
            node,
            covering_node_from(nodes_of_the_level, child, 0),
            child,
        );
    }
}

/// 形状良好时，每一片容器在每一层上都有祖先：它落在那一层范围内，罩住它在下一层的祖先（层下标 0 时就是它自己）。
proof fn lemma_ancestor_index(levels: Seq<Seq<usize>>, level_index: nat, container: int)
    requires
        container_tree_shape_is_well_formed(levels),
        level_index < levels.len(),
        0 <= container < container_count(levels),
    ensures
        0 <= ancestor_index(levels, level_index, container) < levels[level_index as int].len(),
        child_lies_under_node(
            levels[level_index as int],
            ancestor_index(levels, level_index, container),
            if level_index == 0 {
                container
            } else {
                ancestor_index(levels, (level_index - 1) as nat, container)
            },
        ),
    decreases level_index,
{
    if level_index == 0 {
        lemma_covering_node(levels[0], container);
    } else {
        let below = (level_index - 1) as nat;
        lemma_ancestor_index(levels, below, container);
        assert(children_count_sum(levels[level_index as int]) == levels[below as int].len());
        lemma_covering_node(levels[level_index as int], ancestor_index(levels, below, container));
    }
}

/// 形状良好时，每一层最右那个节点都是最后一片容器的祖先（最右那一条路径）。
proof fn lemma_the_rightmost_path_holds_the_last_container(levels: Seq<Seq<usize>>, level_index: nat)
    requires
        container_tree_shape_is_well_formed(levels),
        level_index < levels.len(),
    ensures
        ancestor_index(levels, level_index, container_count(levels) - 1) == levels[level_index as int].len() - 1,
    decreases level_index,
{
    let nodes = levels[level_index as int];
    assert forall|node: int| 0 <= node < nodes.len() implies #[trigger] nodes[node] >= 1 by {
        assert(levels[level_index as int][node] >= 1);
    }
    assert(nodes.len() >= 1);
    lemma_the_rightmost_node_holds_the_last_child(nodes);
    if level_index == 0 {
        lemma_children_count_sum_is_at_least_the_node_count(nodes);
        lemma_covering_node(nodes, container_count(levels) - 1);
    } else {
        let below = (level_index - 1) as nat;
        lemma_the_rightmost_path_holds_the_last_container(levels, below);
        assert(children_count_sum(nodes) == levels[below as int].len());
        lemma_covering_node(nodes, levels[below as int].len() - 1);
    }
}

/// 一层在最右端长出来之后，原来罩住的孩子照样罩住。
proof fn lemma_extended_nodes_keep_their_children(earlier_nodes: Seq<usize>, later_nodes: Seq<usize>, node: int, child: int)
    requires
        nodes_extend(earlier_nodes, later_nodes),
        child_lies_under_node(earlier_nodes, node, child),
    ensures
        child_lies_under_node(later_nodes, node, child),
{
    let last = earlier_nodes.len() - 1;
    assert(later_nodes.take(last) =~= earlier_nodes.take(last));
    if node < last {
        assert(later_nodes.take(node) =~= earlier_nodes.take(node));
        assert(later_nodes.take(node + 1) =~= earlier_nodes.take(node + 1));
    } else {
        lemma_children_count_sum_of_one_more(earlier_nodes, last);
        lemma_children_count_sum_of_one_more(later_nodes, last);
    }
}

/// 形状在最右端长出来之后，原有的容器在原有层上的祖先不变。
proof fn lemma_extended_levels_keep_their_ancestors(
    earlier_levels: Seq<Seq<usize>>,
    later_levels: Seq<Seq<usize>>,
    level_index: nat,
    container: int,
)
    requires
        container_tree_shape_is_well_formed(earlier_levels),
        levels_extend(earlier_levels, later_levels),
        level_index < earlier_levels.len(),
        0 <= container < container_count(earlier_levels),
    ensures
        ancestor_index(later_levels, level_index, container) == ancestor_index(earlier_levels, level_index, container),
    decreases level_index,
{
    lemma_ancestor_index(earlier_levels, level_index, container);
    assert(nodes_extend(earlier_levels[level_index as int], later_levels[level_index as int]));
    let child = if level_index == 0 {
        container
    } else {
        ancestor_index(earlier_levels, (level_index - 1) as nat, container)
    };
    if level_index > 0 {
        lemma_ancestor_index(earlier_levels, (level_index - 1) as nat, container);
        lemma_extended_levels_keep_their_ancestors(earlier_levels, later_levels, (level_index - 1) as nat, container);
    }
    let node = ancestor_index(earlier_levels, level_index, container);
    lemma_extended_nodes_keep_their_children(earlier_levels[level_index as int], later_levels[level_index as int], node, child);
    lemma_children_count_prefix_sum_is_monotone(later_levels[level_index as int], node + 1, node + 1);
    lemma_covering_node(later_levels[level_index as int], child);
}

/// 在最右端长出来是传递的。
pub(super) proof fn lemma_levels_extend_transitively(
    earliest_levels: Seq<Seq<usize>>,
    middle_levels: Seq<Seq<usize>>,
    latest_levels: Seq<Seq<usize>>,
)
    requires
        levels_extend(earliest_levels, middle_levels),
        levels_extend(middle_levels, latest_levels),
    ensures
        levels_extend(earliest_levels, latest_levels),
{
    assert forall|level_index: int| 0 <= level_index < earliest_levels.len() implies nodes_extend(
        #[trigger] earliest_levels[level_index],
        latest_levels[level_index],
    ) by {
        assert(nodes_extend(earliest_levels[level_index], middle_levels[level_index]));
        assert(nodes_extend(middle_levels[level_index], latest_levels[level_index]));
        let last = earliest_levels[level_index].len() - 1;
        assert forall|node: int| 0 <= node < last implies #[trigger] latest_levels[level_index][node]
            == earliest_levels[level_index][node] by {
            assert(middle_levels[level_index][node] == earliest_levels[level_index][node]);
        }
        if last < middle_levels[level_index].len() - 1 {
            assert(latest_levels[level_index][last] == middle_levels[level_index][last]);
        }
    }
}

/// 一路往上找停的那一层：停的那一层之下每层最右节点都满；不是最高那一层时停的那一层装得下。
pub(super) proof fn lemma_opening_stop_level(levels: Seq<Seq<usize>>, entries_per_node: int, level_index: int)
    requires
        0 <= level_index < levels.len(),
    ensures
        level_index <= opening_stop_level_from(levels, entries_per_node, level_index) < levels.len(),
        forall|full_level_index: int|
            level_index <= full_level_index < opening_stop_level_from(levels, entries_per_node, level_index) ==> (
            #[trigger] levels[full_level_index]).last() >= entries_per_node,
        opening_stop_level_from(levels, entries_per_node, level_index) + 1 < levels.len() ==> levels[opening_stop_level_from(
            levels,
            entries_per_node,
            level_index,
        )].last() < entries_per_node,
    decreases levels.len() - level_index,
{
    if !(level_index + 1 >= levels.len() || levels[level_index].last() < entries_per_node) {
        lemma_opening_stop_level(levels, entries_per_node, level_index + 1);
    }
}

/// 最右端新开一片容器之后的一层：孩子数之和、节点数怎么变，在最右端长出来，每个节点至少一个孩子。
proof fn lemma_level_after_opening(levels: Seq<Seq<usize>>, entries_per_node: int, level_index: int)
    requires
        container_tree_shape_is_well_formed(levels),
        entries_per_node <= usize::MAX,
        0 <= level_index < levels.len(),
    ensures
        ({
            let stop = opening_stop_level_from(levels, entries_per_node, 0);
            let grows = levels[stop].last() >= entries_per_node;
            let opened = level_after_opening(levels, stop, grows, level_index);
            &&& children_count_sum(opened) == children_count_sum(levels[level_index]) + (if level_index <= stop {
                1int
            } else {
                0int
            })
            &&& opened.len() == levels[level_index].len() + (if level_index < stop || (level_index == stop && grows) {
                1int
            } else {
                0int
            })
            &&& nodes_extend(levels[level_index], opened)
            &&& forall|node: int| 0 <= node < opened.len() ==> #[trigger] opened[node] >= 1
        }),
{
    lemma_opening_stop_level(levels, entries_per_node, 0);
    let stop = opening_stop_level_from(levels, entries_per_node, 0);
    let grows = levels[stop].last() >= entries_per_node;
    let nodes = levels[level_index];
    let opened = level_after_opening(levels, stop, grows, level_index);
    assert(nodes.len() >= 1);
    assert forall|node: int| 0 <= node < nodes.len() implies #[trigger] nodes[node] >= 1 by {
        assert(levels[level_index][node] >= 1);
    }
    if level_index < stop || (level_index == stop && grows) {
        lemma_children_count_sum_after_a_push(nodes, 1usize);
        if level_index < stop {
            assert(levels[level_index].last() >= entries_per_node);
        }
        assert forall|node: int| 0 <= node < opened.len() implies #[trigger] opened[node] >= 1 by {
            if node < nodes.len() {
                assert(opened[node] == nodes[node]);
            }
        }
    } else if level_index == stop {
        assert(stop + 1 >= levels.len() ==> !grows);
        lemma_children_count_sum_after_an_update_of_the_last(nodes, (nodes.last() + 1) as usize);
        assert forall|node: int| 0 <= node < opened.len() implies #[trigger] opened[node] >= 1 by {
            if node < nodes.len() - 1 {
                assert(opened[node] == nodes[node]);
            }
        }
    }
}

/// 最右端新开一片容器之后形状照样良好、容器数加一。
pub(super) proof fn lemma_opening_keeps_the_shape_well_formed(levels: Seq<Seq<usize>>, entries_per_node: int)
    requires
        container_tree_shape_is_well_formed(levels),
        container_count(levels) + 1 <= usize::MAX,
        2 <= entries_per_node <= usize::MAX,
    ensures
        container_tree_shape_is_well_formed(levels_after_opening_at_the_right_end(levels, entries_per_node)),
        container_count(levels_after_opening_at_the_right_end(levels, entries_per_node)) == container_count(levels) + 1,
{
    let opened = levels_after_opening_at_the_right_end(levels, entries_per_node);
    if levels.len() == 0 {
        lemma_children_count_sum_of_a_single_node(1usize);
        assert(opened[0] =~= seq![1usize]);
    } else {
        lemma_opening_stop_level(levels, entries_per_node, 0);
        let stop = opening_stop_level_from(levels, entries_per_node, 0);
        let grows = levels[stop].last() >= entries_per_node;
        assert(grows ==> stop == levels.len() - 1);
        assert forall|level_index: int| 0 <= level_index < levels.len() implies #[trigger] opened[level_index]
            == level_after_opening(levels, stop, grows, level_index) by {}
        if grows {
            lemma_children_count_sum_of_a_single_node(2usize);
            assert(opened[levels.len() as int] =~= seq![2usize]);
        }
        assert forall|level_index: int| 0 <= level_index < opened.len() implies (#[trigger] opened[level_index]).len() >= 1
            && forall|node: int| 0 <= node < opened[level_index].len() ==> #[trigger] opened[level_index][node] >= 1 by {
            if level_index < levels.len() {
                lemma_level_after_opening(levels, entries_per_node, level_index);
            }
        }
        assert forall|upper_level_index: int| 1 <= upper_level_index < opened.len() implies children_count_sum(
            #[trigger] opened[upper_level_index],
        ) == opened[upper_level_index - 1].len() by {
            lemma_level_after_opening(levels, entries_per_node, upper_level_index - 1);
            if upper_level_index < levels.len() {
                lemma_level_after_opening(levels, entries_per_node, upper_level_index);
                assert(children_count_sum(levels[upper_level_index]) == levels[upper_level_index - 1].len());
            } else {
                assert(levels[levels.len() - 1].len() == 1);
            }
        }
        lemma_level_after_opening(levels, entries_per_node, 0);
        lemma_level_after_opening(levels, entries_per_node, levels.len() - 1);
        assert(levels[levels.len() - 1].len() == 1);
    }
}

/// 最右端新开一片容器之后形状在最右端长出来；新出现的节点都在它那一层的最右端。
pub(super) proof fn lemma_opening_extends_the_levels_at_the_right_end(levels: Seq<Seq<usize>>, entries_per_node: int)
    requires
        container_tree_shape_is_well_formed(levels),
        2 <= entries_per_node <= usize::MAX,
    ensures
        levels_extend(levels, levels_after_opening_at_the_right_end(levels, entries_per_node)),
        forall|level_index: int, node: int|
            #[trigger] node_is_in_the_levels(levels_after_opening_at_the_right_end(levels, entries_per_node), level_index, node)
                && !node_is_in_the_levels(levels, level_index, node) ==> node
                == levels_after_opening_at_the_right_end(levels, entries_per_node)[level_index].len() - 1,
{
    let opened = levels_after_opening_at_the_right_end(levels, entries_per_node);
    if levels.len() == 0 {
        assert(opened[0] =~= seq![1usize]);
    } else {
        lemma_opening_stop_level(levels, entries_per_node, 0);
        let stop = opening_stop_level_from(levels, entries_per_node, 0);
        let grows = levels[stop].last() >= entries_per_node;
        assert forall|level_index: int| 0 <= level_index < levels.len() implies #[trigger] opened[level_index]
            == level_after_opening(levels, stop, grows, level_index) by {}
        if grows {
            assert(opened[levels.len() as int] =~= seq![2usize]);
        }
        assert forall|level_index: int| 0 <= level_index < levels.len() implies nodes_extend(
            #[trigger] levels[level_index],
            opened[level_index],
        ) by {
            lemma_level_after_opening(levels, entries_per_node, level_index);
        }
        assert forall|level_index: int, node: int|
            #[trigger] node_is_in_the_levels(opened, level_index, node) && !node_is_in_the_levels(levels, level_index, node) implies node
                == opened[level_index].len() - 1 by {
            if level_index < levels.len() {
                lemma_level_after_opening(levels, entries_per_node, level_index);
            }
        }
    }
}

/// 多重写几片，罩住的照样罩住。
pub(super) proof fn lemma_more_rewritten_containers_keep_every_new_node_rewritten(
    previous_node_counts: Seq<int>,
    levels: Seq<Seq<usize>>,
    rewritten_containers: Seq<usize>,
    added: usize,
)
    requires
        every_node_absent_from_the_previous_version_is_rewritten(previous_node_counts, levels, rewritten_containers),
    ensures
        every_node_absent_from_the_previous_version_is_rewritten(previous_node_counts, levels, rewritten_containers.push(added)),
{
    let longer = rewritten_containers.push(added);
    assert forall|level_index: int, node: int|
        #[trigger] node_is_in_the_levels(levels, level_index, node) && !node_is_counted(
            previous_node_counts,
            level_index,
            node,
        ) implies node_is_an_ancestor_of_one_of(levels, level_index, node, longer) by {
        let listed = choose|listed: int|
            0 <= listed < rewritten_containers.len() && #[trigger] node_is_an_ancestor_of_container(
                levels,
                level_index as nat,
                node,
                rewritten_containers[listed] as int,
            );
        assert(longer[listed] == rewritten_containers[listed]);
    }
}

/// 最右端新开一片容器、把它记进重写清单之后，新出现的节点照样都罩着重写的某一片。
pub(super) proof fn lemma_opening_keeps_every_new_node_rewritten(
    previous_node_counts: Seq<int>,
    levels: Seq<Seq<usize>>,
    entries_per_node: int,
    rewritten_containers: Seq<usize>,
)
    requires
        container_tree_shape_is_well_formed(levels),
        container_count(levels) + 1 <= usize::MAX,
        2 <= entries_per_node <= usize::MAX,
        forall|listed: int| 0 <= listed < rewritten_containers.len() ==> #[trigger] rewritten_containers[listed] < container_count(levels),
        every_node_absent_from_the_previous_version_is_rewritten(previous_node_counts, levels, rewritten_containers),
    ensures
        every_node_absent_from_the_previous_version_is_rewritten(
            previous_node_counts,
            levels_after_opening_at_the_right_end(levels, entries_per_node),
            rewritten_containers.push(container_count(levels) as usize),
        ),
{
    let opened = levels_after_opening_at_the_right_end(levels, entries_per_node);
    let longer = rewritten_containers.push(container_count(levels) as usize);
    if levels.len() > 0 {
        lemma_children_count_sum_is_not_negative(levels[0]);
    }
    lemma_opening_keeps_the_shape_well_formed(levels, entries_per_node);
    lemma_opening_extends_the_levels_at_the_right_end(levels, entries_per_node);
    assert forall|level_index: int, node: int|
        #[trigger] node_is_in_the_levels(opened, level_index, node) && !node_is_counted(
            previous_node_counts,
            level_index,
            node,
        ) implies node_is_an_ancestor_of_one_of(opened, level_index, node, longer) by {
        if node_is_in_the_levels(levels, level_index, node) {
            let listed = choose|listed: int|
                0 <= listed < rewritten_containers.len() && #[trigger] node_is_an_ancestor_of_container(
                    levels,
                    level_index as nat,
                    node,
                    rewritten_containers[listed] as int,
                );
            lemma_extended_levels_keep_their_ancestors(levels, opened, level_index as nat, rewritten_containers[listed] as int);
            assert(longer[listed] == rewritten_containers[listed]);
            assert(node_is_an_ancestor_of_container(opened, level_index as nat, node, longer[listed] as int));
        } else {
            assert(node == opened[level_index].len() - 1);
            lemma_the_rightmost_path_holds_the_last_container(opened, level_index as nat);
            assert(longer[rewritten_containers.len() as int] == container_count(levels));
            assert(node_is_an_ancestor_of_container(
                opened,
                level_index as nat,
                node,
                longer[rewritten_containers.len() as int] as int,
            ));
        }
    }
}

/// 一个节点是不是 `containers` 某一片的祖先，就看它罩住的孩子里有没有被标过的。
pub(super) proof fn lemma_node_is_marked_by_its_children(levels: Seq<Seq<usize>>, level_index: int, node: int, containers: Seq<usize>)
    requires
        container_tree_shape_is_well_formed(levels),
        0 <= level_index < levels.len(),
        forall|listed: int| 0 <= listed < containers.len() ==> #[trigger] containers[listed] < container_count(levels),
    ensures
        node_is_an_ancestor_of_one_of(levels, level_index, node, containers) <==> exists|child: int|
            child_lies_under_node(levels[level_index], node, child) && #[trigger] child_is_marked(
                levels,
                level_index,
                child,
                containers,
            ),
{
    if node_is_an_ancestor_of_one_of(levels, level_index, node, containers) {
        let listed = choose|listed: int|
            0 <= listed < containers.len() && #[trigger] node_is_an_ancestor_of_container(
                levels,
                level_index as nat,
                node,
                containers[listed] as int,
            );
        lemma_ancestor_index(levels, level_index as nat, containers[listed] as int);
        let child = if level_index == 0 {
            containers[listed] as int
        } else {
            ancestor_index(levels, (level_index - 1) as nat, containers[listed] as int)
        };
        assert(child_lies_under_node(levels[level_index], node, child));
        if level_index > 0 {
            assert(node_is_an_ancestor_of_container(levels, (level_index - 1) as nat, child, containers[listed] as int));
        }
        assert(child_is_marked(levels, level_index, child, containers));
    }
    if exists|child: int|
        child_lies_under_node(levels[level_index], node, child) && #[trigger] child_is_marked(
            levels,
            level_index,
            child,
            containers,
        ) {
        let child = choose|child: int|
            child_lies_under_node(levels[level_index], node, child) && #[trigger] child_is_marked(
                levels,
                level_index,
                child,
                containers,
            );
        lemma_a_node_over_a_marked_child_is_an_ancestor(levels, level_index, node, child, containers);
    }
}

/// 一个节点罩住的孩子里有一个被标过，它就是 `containers` 某一片的祖先：[`lemma_node_is_marked_by_its_children`] 从孩子推到节点的那一半，
/// 单列一个引理，那一个的查询只剩从节点推到孩子那一半（整 crate 一起验时两半挤在一个查询里，在登记的 rlimit 上验不过）。
proof fn lemma_a_node_over_a_marked_child_is_an_ancestor(
    levels: Seq<Seq<usize>>,
    level_index: int,
    node: int,
    child: int,
    containers: Seq<usize>,
)
    requires
        container_tree_shape_is_well_formed(levels),
        0 <= level_index < levels.len(),
        forall|listed: int| 0 <= listed < containers.len() ==> #[trigger] containers[listed] < container_count(levels),
        child_lies_under_node(levels[level_index], node, child),
        child_is_marked(levels, level_index, child, containers),
    ensures
        node_is_an_ancestor_of_one_of(levels, level_index, node, containers),
{
    lemma_children_count_prefix_sum_is_monotone(levels[level_index], node + 1, node + 1);
    lemma_children_count_sum_is_not_negative(levels[level_index].take(node));
    lemma_covering_node(levels[level_index], child);
    // 两支都在 `node_is_an_ancestor_of_one_of` 自己的触发式（层下标写成 `level_index as nat`）上给出见证。
    if level_index == 0 {
        let listed = choose|listed: int| 0 <= listed < containers.len() && #[trigger] containers[listed] == child;
        assert(node_is_an_ancestor_of_container(levels, level_index as nat, node, containers[listed] as int));
        assert(0 <= listed < containers.len());
    } else {
        let listed = choose|listed: int|
            0 <= listed < containers.len() && #[trigger] node_is_an_ancestor_of_container(
                levels,
                (level_index - 1) as nat,
                child,
                containers[listed] as int,
            );
        assert(node_is_an_ancestor_of_container(levels, level_index as nat, node, containers[listed] as int));
        assert(0 <= listed < containers.len());
    }
}

/// 两段接起来，含不含某一样就看两段各含不含。
pub(super) proof fn lemma_concatenation_contains<Element>(earlier: Seq<Element>, later: Seq<Element>, element: Element)
    ensures
        (earlier + later).contains(element) <==> (earlier.contains(element) || later.contains(element)),
{
    let joined = earlier + later;
    if joined.contains(element) {
        let found = choose|found: int| 0 <= found < joined.len() && joined[found] == element;
        if found < earlier.len() {
            assert(earlier[found] == element);
        } else {
            assert(later[found - earlier.len()] == element);
        }
    }
    if earlier.contains(element) {
        let found = choose|found: int| 0 <= found < earlier.len() && earlier[found] == element;
        assert(joined[found] == element);
    }
    if later.contains(element) {
        let found = choose|found: int| 0 <= found < later.len() && later[found] == element;
        assert(joined[earlier.len() + found] == element);
    }
}

/// 一层里的祖先位置都在这一层上、序号小于前几个的个数，且「在这张单子里」就是「是某一片的祖先」。
pub(super) proof fn lemma_ancestors_in_level(
    levels: Seq<Seq<usize>>,
    level_index: int,
    containers: Seq<usize>,
    node_count: int,
)
    requires
        0 <= level_index < 255,
        0 <= node_count <= u32::MAX + 1,
    ensures
        forall|listed: int|
            0 <= listed < ancestors_in_level(levels, level_index, containers, node_count).len() ==> {
                let position = #[trigger] ancestors_in_level(levels, level_index, containers, node_count)[listed];
                &&& position.level == level_index + 1
                &&& position.index_in_level < node_count
            },
        forall|node: int|
            0 <= node < node_count ==> (ancestors_in_level(levels, level_index, containers, node_count).contains(
                #[trigger] position_at(level_index, node),
            ) <==> node_is_an_ancestor_of_one_of(levels, level_index, node, containers)),
    decreases node_count,
{
    if node_count > 0 {
        lemma_ancestors_in_level(levels, level_index, containers, node_count - 1);
        let earlier = ancestors_in_level(levels, level_index, containers, node_count - 1);
        let last = position_at(level_index, node_count - 1);
        let listed_here = ancestors_in_level(levels, level_index, containers, node_count);
        let last_is_listed = node_is_an_ancestor_of_one_of(levels, level_index, node_count - 1, containers);
        assert(listed_here == if last_is_listed {
            earlier.push(last)
        } else {
            earlier
        });
        assert(earlier.push(last) =~= earlier + seq![last]);
        assert forall|node: int| 0 <= node < node_count implies (listed_here.contains(#[trigger] position_at(level_index, node))
            <==> node_is_an_ancestor_of_one_of(levels, level_index, node, containers)) by {
            let position = position_at(level_index, node);
            lemma_concatenation_contains(earlier, seq![last], position);
            if node < node_count - 1 {
                assert(position.index_in_level != last.index_in_level);
                assert(!seq![last].contains(position)) by {
                    if seq![last].contains(position) {
                        let found = choose|found: int| 0 <= found < 1 && seq![last][found] == position;
                    }
                }
            } else {
                assert(seq![last][0] == last);
                if earlier.contains(position) {
                    let found = choose|found: int| 0 <= found < earlier.len() && earlier[found] == position;
                    assert(earlier[found].index_in_level < node_count - 1);
                }
            }
        }
        assert forall|listed: int| 0 <= listed < listed_here.len() implies {
            let position = #[trigger] listed_here[listed];
            &&& position.level == level_index + 1
            &&& position.index_in_level < node_count
        } by {
            if listed < earlier.len() {
                assert(listed_here[listed] == earlier[listed]);
            }
        }
    }
}

/// 一层前 `node_count` 个节点都不是 `containers` 某一片的祖先时，那几个里的祖先位置是空的。
pub(super) proof fn lemma_no_ancestor_among_the_first(levels: Seq<Seq<usize>>, level_index: int, containers: Seq<usize>, node_count: int)
    requires
        forall|node: int|
            0 <= node < node_count ==> !(#[trigger] node_is_an_ancestor_of_one_of(levels, level_index, node, containers)),
    ensures
        ancestors_in_level(levels, level_index, containers, node_count) == Seq::<ContainerTreeNodePosition>::empty(),
    decreases node_count,
{
    if node_count > 0 {
        assert(!node_is_an_ancestor_of_one_of(levels, level_index, node_count - 1, containers));
        lemma_no_ancestor_among_the_first(levels, level_index, containers, node_count - 1);
    }
}

/// 最右端那 `count` 片里有没有第 `child` 片：有就是它落在最右端那几片的头一片到最右那片之间（一共 `total` 片）。
pub(super) proof fn lemma_rightmost_containers_hold_exactly_the_ones_from_their_first(total: int, count: int, child: int)
    requires
        0 <= count <= total <= usize::MAX,
    ensures
        (exists|listed: int|
            0 <= listed < rightmost_containers(total, count).len() && #[trigger] rightmost_containers(total, count)[listed] == child)
            <==> (total - count <= child < total),
{
    let containers = rightmost_containers(total, count);
    if total - count <= child < total {
        let listed = child - (total - count);
        assert(containers[listed] == child);
    }
    if exists|listed: int| 0 <= listed < containers.len() && #[trigger] containers[listed] == child {
        let listed = choose|listed: int| 0 <= listed < containers.len() && #[trigger] containers[listed] == child;
        assert(containers[listed] == total - count + listed);
    }
}

/// 重写清单是最右端那几片时，它就是 [`rightmost_containers`] 那一份。
pub(super) proof fn lemma_rewritten_containers_are_the_rightmost_ones(rewritten_containers: Seq<usize>, total: int)
    requires
        rewritten_containers_are_the_rightmost(rewritten_containers, total),
        total <= usize::MAX,
    ensures
        rewritten_containers == rightmost_containers(total, rewritten_containers.len() as int),
{
    assert(rewritten_containers =~= rightmost_containers(total, rewritten_containers.len() as int));
}

/// 一片都没给时哪一层都没有祖先。
pub(super) proof fn lemma_no_containers_have_no_ancestors(levels: Seq<Seq<usize>>, level_count: int)
    ensures
        ancestors_below_level(levels, Seq::<usize>::empty(), level_count) == Seq::<ContainerTreeNodePosition>::empty(),
    decreases level_count,
{
    if level_count > 0 {
        lemma_no_containers_have_no_ancestors(levels, level_count - 1);
        assert forall|node: int| 0 <= node < levels[level_count - 1].len() implies !(#[trigger] node_is_an_ancestor_of_one_of(
            levels,
            level_count - 1,
            node,
            Seq::<usize>::empty(),
        )) by {}
        lemma_no_ancestor_among_the_first(levels, level_count - 1, Seq::<usize>::empty(), levels[level_count - 1].len() as int);
        assert(Seq::<ContainerTreeNodePosition>::empty() + Seq::<ContainerTreeNodePosition>::empty()
            =~= Seq::<ContainerTreeNodePosition>::empty());
    }
}

/// 装得进位置两段的形状里，位置的两段就是层级与序号。
proof fn lemma_position_at_fits(levels: Seq<Seq<usize>>, level_index: int, node: int)
    requires
        shape_fits_the_position_fields(levels),
        container_tree_shape_is_well_formed(levels),
        node_is_in_the_levels(levels, level_index, node),
    ensures
        position_at(level_index, node).level == level_index + 1,
        position_at(level_index, node).index_in_level == node,
{
    lemma_every_level_has_at_most_as_many_nodes_as_containers(levels, level_index);
}

/// 一片接一片追加记录，记录依次连起来就是原来的接上这一条。
pub(super) proof fn lemma_records_of_containers_after_a_push<Record>(containers: Seq<RecordContainer<Record>>, container: RecordContainer<Record>)
    ensures
        records_of_containers(containers.push(container)) == records_of_containers(containers) + container.records@,
{
    assert(containers.push(container).drop_last() =~= containers);
}

/// 最右那片换成记录多了一条的那一份，记录依次连起来就是原来的接上这一条。
pub(super) proof fn lemma_records_of_containers_after_appending_to_the_last<Record>(
    containers: Seq<RecordContainer<Record>>,
    appended_to: Seq<RecordContainer<Record>>,
    record: Record,
)
    requires
        containers.len() >= 1,
        appended_to.len() == containers.len(),
        appended_to.drop_last() == containers.drop_last(),
        appended_to.last().records@ == containers.last().records@.push(record),
    ensures
        records_of_containers(appended_to) == records_of_containers(containers).push(record),
{
    assert(records_of_containers(containers).push(record) =~= records_of_containers(containers.drop_last())
        + containers.last().records@.push(record));
}

/// 逐层逐个照抄孩子数。
pub(super) fn copy_of_the_children_counts(children_of_each_node_at_each_level: &[Vec<usize>]) -> (copy: Vec<Vec<usize>>)
    ensures
        levels_of_children_counts(copy@) == levels_of_children_counts(children_of_each_node_at_each_level@),
{
    let mut copy: Vec<Vec<usize>> = Vec::new();
    let mut level_index: usize = 0;
    while level_index < children_of_each_node_at_each_level.len()
        invariant
            level_index <= children_of_each_node_at_each_level.len(),
            copy.len() == level_index,
            forall|copied_level_index: int|
                0 <= copied_level_index < level_index ==> (#[trigger] copy@[copied_level_index])@
                    == children_of_each_node_at_each_level@[copied_level_index]@,
        decreases children_of_each_node_at_each_level.len() - level_index,
    {
        copy.push(copy_of_one_level(&children_of_each_node_at_each_level[level_index]));
        level_index += 1;
    }
    assert(levels_of_children_counts(copy@) =~= levels_of_children_counts(children_of_each_node_at_each_level@));
    copy
}

/// 照抄一层的孩子数。
fn copy_of_one_level(children_of_each_node: &[usize]) -> (copy: Vec<usize>)
    ensures
        copy@ == children_of_each_node@,
{
    let mut copy: Vec<usize> = Vec::new();
    let mut node: usize = 0;
    while node < children_of_each_node.len()
        invariant
            node <= children_of_each_node.len(),
            copy@ == children_of_each_node@.take(node as int),
        decreases children_of_each_node.len() - node,
    {
        copy.push(children_of_each_node[node]);
        node += 1;
        assert(copy@ =~= children_of_each_node@.take(node as int));
    }
    assert(children_of_each_node@.take(children_of_each_node.len() as int) =~= children_of_each_node@);
    copy
}

/// 只有一个孩子数的一层。
pub(super) fn level_of_a_single_node(children: usize) -> (level: Vec<usize>)
    ensures
        level@ == seq![children],
{
    let level = vec![children];
    assert(level@ =~= seq![children]);
    level
}

/// 一层孩子数之和；越过 `usize` 交回 `None`。
pub(super) fn checked_children_count_sum(children_of_each_node: &[usize]) -> (sum: Option<usize>)
    ensures
        sum is Some <==> children_count_sum(children_of_each_node@) <= usize::MAX,
        sum matches Some(total) ==> total == children_count_sum(children_of_each_node@),
{
    let mut total: usize = 0;
    let mut node: usize = 0;
    while node < children_of_each_node.len()
        invariant
            node <= children_of_each_node.len(),
            total == children_count_sum(children_of_each_node@.take(node as int)),
        decreases children_of_each_node.len() - node,
    {
        proof {
            lemma_children_count_sum_of_one_more(children_of_each_node@, node as int);
        }
        match total.checked_add(children_of_each_node[node]) {
            Some(next_total) => {
                total = next_total;
            },
            None => {
                proof {
                    lemma_children_count_prefix_sum_is_monotone(children_of_each_node@, node as int + 1, node as int + 1);
                }
                return None;
            },
        }
        node += 1;
    }
    proof {
        lemma_children_count_sum_of_the_whole_level(children_of_each_node@);
    }
    Some(total)
}

/// 每一层至少一个节点、每个节点至少一个孩子。
pub(super) fn every_node_has_a_child(children_of_each_node_at_each_level: &[Vec<usize>]) -> (every_node_has_a_child: bool)
    ensures
        every_node_has_a_child == every_level_has_a_node_and_every_node_a_child(
            levels_of_children_counts(children_of_each_node_at_each_level@),
        ),
{
    let ghost levels = levels_of_children_counts(children_of_each_node_at_each_level@);
    let mut level_index: usize = 0;
    while level_index < children_of_each_node_at_each_level.len()
        invariant
            levels == levels_of_children_counts(children_of_each_node_at_each_level@),
            levels.len() == children_of_each_node_at_each_level.len(),
            level_index <= levels.len(),
            forall|checked_level_index: int|
                0 <= checked_level_index < level_index ==> (#[trigger] levels[checked_level_index]).len() >= 1,
            forall|checked_level_index: int, node: int|
                0 <= checked_level_index < level_index && 0 <= node < levels[checked_level_index].len()
                    ==> #[trigger] levels[checked_level_index][node] >= 1,
        decreases children_of_each_node_at_each_level.len() - level_index,
    {
        let nodes = &children_of_each_node_at_each_level[level_index];
        proof {
            assert(levels[level_index as int] == nodes@);
        }
        if nodes.is_empty() {
            proof {
                assert(levels[level_index as int].len() == 0);
            }
            return false;
        }
        let mut node: usize = 0;
        while node < nodes.len()
            invariant
                levels == levels_of_children_counts(children_of_each_node_at_each_level@),
                levels.len() == children_of_each_node_at_each_level.len(),
                node <= nodes.len(),
                level_index < levels.len(),
                levels[level_index as int] == nodes@,
                forall|checked_node: int| 0 <= checked_node < node ==> #[trigger] nodes@[checked_node] >= 1,
                forall|checked_level_index: int|
                    0 <= checked_level_index < level_index ==> (#[trigger] levels[checked_level_index]).len() >= 1,
                forall|checked_level_index: int, checked_node: int|
                    0 <= checked_level_index < level_index && 0 <= checked_node < levels[checked_level_index].len()
                        ==> #[trigger] levels[checked_level_index][checked_node] >= 1,
            decreases nodes.len() - node,
        {
            if nodes[node] == 0 {
                proof {
                    assert(levels[level_index as int][node as int] == 0);
                }
                return false;
            }
            node += 1;
        }
        proof {
            assert forall|checked_level_index: int, checked_node: int|
                0 <= checked_level_index < level_index + 1 && 0 <= checked_node < levels[checked_level_index].len()
                    implies #[trigger] levels[checked_level_index][checked_node] >= 1 by {
                if checked_level_index == level_index {
                    assert(nodes@[checked_node] >= 1);
                }
            }
        }
        level_index += 1;
    }
    true
}

/// 层级 ℓ > 1 的孩子数之和都等于层级 ℓ − 1 的节点数（算和越过 `usize` 的那一层算不等）。
pub(super) fn each_level_counts_the_nodes_below(children_of_each_node_at_each_level: &[Vec<usize>]) -> (counts_the_nodes_below: bool)
    ensures
        counts_the_nodes_below == each_upper_level_counts_the_nodes_below(
            levels_of_children_counts(children_of_each_node_at_each_level@),
        ),
{
    let ghost levels = levels_of_children_counts(children_of_each_node_at_each_level@);
    let mut upper_level_index: usize = 1;
    while upper_level_index < children_of_each_node_at_each_level.len()
        invariant
            levels == levels_of_children_counts(children_of_each_node_at_each_level@),
            levels.len() == children_of_each_node_at_each_level.len(),
            1 <= upper_level_index,
            forall|checked_level_index: int|
                1 <= checked_level_index < upper_level_index && checked_level_index < levels.len() ==> children_count_sum(
                    #[trigger] levels[checked_level_index],
                ) == levels[checked_level_index - 1].len(),
        decreases children_of_each_node_at_each_level.len() - upper_level_index,
    {
        let nodes_below = children_of_each_node_at_each_level[upper_level_index - 1].len();
        proof {
            assert(levels[upper_level_index as int] == children_of_each_node_at_each_level@[upper_level_index as int]@);
            assert(levels[upper_level_index - 1] == children_of_each_node_at_each_level@[upper_level_index - 1]@);
        }
        match checked_children_count_sum(&children_of_each_node_at_each_level[upper_level_index]) {
            Some(sum) => {
                if sum != nodes_below {
                    return false;
                }
            },
            None => {
                return false;
            },
        }
        upper_level_index += 1;
    }
    true
}

/// 这个位置在不在这张单子里。
pub(super) fn position_is_listed(positions: &[ContainerTreeNodePosition], position: ContainerTreeNodePosition) -> (is_listed: bool)
    ensures
        is_listed == positions@.contains(position),
{
    let mut listed: usize = 0;
    while listed < positions.len()
        invariant
            listed <= positions.len(),
            forall|checked: int| 0 <= checked < listed ==> #[trigger] positions@[checked] != position,
        decreases positions.len() - listed,
    {
        if positions[listed].level == position.level && positions[listed].index_in_level == position.index_in_level {
            proof {
                assert(positions@[listed as int] == position);
            }
            return true;
        }
        listed += 1;
    }
    false
}

} // verus!
