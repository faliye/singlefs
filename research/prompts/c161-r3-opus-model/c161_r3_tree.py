#!/usr/bin/env python3
"""C161（缓冲两级并存而衔接没定） 第三轮攻方腿 · A 部分：带重组的 Bε 树。

三个 key（a < b < c）、树高 1–4、内部节点扇出 ≤ 2、每个缓冲 ≤ 2 条消息。
操作：前端写 / 删、甲的不走前端的写 / 删、flush（一次一条）、部分下推（一个节点 → 一个孩子）、
叶分裂 / 内部节点分裂（在根上即长高）、兄弟合并、塌缩（根只剩一个孩子）、再平衡（叶挪一个 key 位、
内部节点挪一个孩子）、索引重建（D11 已定项 5：先排空再重建成单叶）、发布、崩溃。
判据：U1 每个可达状态每个 key 读一遍比真值；G1 每次重组前后每个 key 读出来的值不变；
G2 内部节点缓冲里只有归它管的 key；G3 丙的前端树叶上的逻辑改动都对得上这次排空的前端条目。
只用标准库；穷举 + 固定种子的随机游走，确定性。"""
import collections
import random
import sys

KEY_COUNT = 3
KEY_NAMES = 'abc'
MAX_FANOUT = 2
BUFFER_CAPACITY = 2
MAX_HEIGHT = 4
STRUCTURAL = ('split_leaf', 'split_inner', 'merge', 'collapse', 'rebalance', 'rebuild')

# 节点：叶 ('L', lo, hi, ((key, version), ...))；内部节点 ('I', lo, hi, 缓冲, 孩子)。区间 [lo, hi) 按 key 下标。
# 消息与前端条目：(key, version, is_delete, seq)；缓冲按到达位置排，越靠后越新。
# 状态：(树, 前端, 计数器, 盘上树, 盘上计数器, 真值, 盘上真值, 本窗口排空到叶的条目)；真值每个 key 一个 (version, is_delete)。


def is_leaf(node):
    return node[0] == 'L'


def make_leaf(low, high, items):
    return ('L', low, high, tuple(sorted(items)))


def make_inner(low, high, buffer, children):
    return ('I', low, high, tuple(buffer), tuple(children))


def with_buffer(node, buffer):
    return (node[0], node[1], node[2], tuple(buffer), node[4])


def with_children(node, children):
    return (node[0], node[1], node[2], node[3], tuple(children))


def node_at(tree, path):
    node = tree
    for index in path:
        node = node[4][index]
    return node


def replace_at(tree, path, new_node):
    if not path:
        return new_node
    children = list(tree[4])
    children[path[0]] = replace_at(children[path[0]], path[1:], new_node)
    return with_children(tree, children)


def all_paths(tree, path=()):
    yield path, tree
    if not is_leaf(tree):
        for index, child in enumerate(tree[4]):
            yield from all_paths(child, path + (index,))


def tree_height(tree):
    return 1 if is_leaf(tree) else 1 + tree_height(tree[4][0])


def child_index_for_key(node, key):
    for index, child in enumerate(node[4]):
        if child[1] <= key < child[2]:
            return index
    raise AssertionError('key 不在任何孩子的区间里：区间划分坏了')


def leaf_path_for_key(tree, key):
    path, node = (), tree
    while not is_leaf(node):
        index = child_index_for_key(node, key)
        path, node = path + (index,), node[4][index]
    return path


def path_nodes_for_key(tree, key):
    nodes, node = [tree], tree
    while not is_leaf(node):
        node = node[4][child_index_for_key(node, key)]
        nodes.append(node)
    return nodes


def leaf_items(tree):
    items = {}
    for _, node in all_paths(tree):
        if is_leaf(node):
            items.update(dict(node[3]))
    return items


def keys_mentioned(tree):
    """子树里叶上有的 key 与各层缓冲里提到的 key（「随 key 搬走」最弱读法认的那一批）。"""
    keys = set()
    for _, node in all_paths(tree):
        keys.update(item[0] for item in node[3])
    return keys


def visible(message):
    return None if message[2] else message[1]


def insert_messages(arm, buffer, messages):
    """甲 / 乙-G：追加在后面（位置即新旧）；乙-D：同 key 至多一条，新来的替换旧的。"""
    buffer = list(buffer)
    for message in messages:
        if arm['kind'] == 'yiD':
            buffer = [existing for existing in buffer if existing[0] != message[0]]
        buffer.append(message)
    return tuple(buffer)


def apply_to_leaf(arm, leaf, messages):
    """按位置依次施加；乙-G 同一批里同 key 取 seq 最大的一条，同 seq 而值不同 ⇒ None（读法给两个答案）。"""
    items = dict(leaf[3])
    if arm['kind'] == 'yiG':
        groups = collections.OrderedDict()
        for message in messages:
            groups.setdefault(message[0], []).append(message)
        chosen = []
        for group in groups.values():
            top_seq = max(message[3] for message in group)
            tops = [message for message in group if message[3] == top_seq]
            if len({(message[1], message[2]) for message in tops}) > 1:
                return None
            chosen.append(tops[0])
        messages = chosen
    for message in messages:
        if message[2]:
            items.pop(message[0], None)
        else:
            items[message[0]] = message[1]
    return make_leaf(leaf[1], leaf[2], items.items())


def read_key(arm, state, key):
    """读法「前端 → 根缓冲 → … → 叶」取第一条；返回可见值的集合，多于一个 = 读法给两个答案。"""
    for entry in reversed(state[1]):
        if entry[0] == key:
            return frozenset([visible(entry)])
    for node in path_nodes_for_key(state[0], key):
        if is_leaf(node):
            return frozenset([dict(node[3]).get(key)])
        candidates = [message for message in node[3] if message[0] == key]
        if not candidates:
            continue
        if arm['kind'] == 'yiG':                       # 同一层由 seq 定新旧
            top_seq = max(message[3] for message in candidates)
            return frozenset(visible(message) for message in candidates if message[3] == top_seq)
        return frozenset([visible(candidates[-1])])    # 位置：越靠后越新
    raise AssertionError('路径没有走到叶')


def all_reads(arm, state):
    return tuple(read_key(arm, state, key) for key in range(KEY_COUNT))


def truth_visible(pair):
    return None if pair[1] else pair[0]


def set_truth(truth, key, version, is_delete):
    truth = list(truth)
    truth[key] = (version, is_delete)
    return tuple(truth)


def iter_versions(state):
    tree, frontend, _, snap_tree, _, truth, snap_truth, applied = state
    for entry in frontend:
        yield entry[0], entry[1]
    for source in (tree, snap_tree):
        for _, node in all_paths(source):
            for item in node[3]:
                yield item[0], item[1]
    for pairs in (truth, snap_truth):
        for key, pair in enumerate(pairs):
            yield key, pair[0]
    for key, value in applied:
        if value is not None:
            yield key, value


def iter_seqs(state):
    tree, frontend, counter, snap_tree, snap_counter, _, _, _ = state
    yield counter
    yield snap_counter
    for entry in frontend:
        yield entry[3]
    for source in (tree, snap_tree):
        for _, node in all_paths(source):
            if not is_leaf(node):
                for message in node[3]:
                    yield message[3]


def next_version(state, key):
    return 1 + max([version for item_key, version in iter_versions(state) if item_key == key] + [0])


def adjacency_preserving_map(values):
    """保序、保「相邻差 1」的重编号：新 seq = 计数器 + 1 撞不撞现有的某个 seq，重编号前后一样。"""
    ordered = sorted(set(values) | {0})
    mapping = {0: 0}
    for previous, value in zip(ordered, ordered[1:]):
        mapping[value] = mapping[previous] + (1 if value - previous == 1 else 2)
    return mapping


def relabel_state(state):
    """状态签名：每个 key 的版本号按大小重编成 0..n，seq 保序保相邻地重编。读对不对只看同 key 的相对次序。"""
    per_key = collections.defaultdict(set)
    for key, version in iter_versions(state):
        per_key[key].add(version)
    version_maps = {key: {version: rank for rank, version in enumerate(sorted(values | {0}))}
                    for key, values in per_key.items()}
    seq_map = adjacency_preserving_map(iter_seqs(state))

    def map_version(key, version):
        return version_maps[key][version]

    def map_message(message):
        return (message[0], map_version(message[0], message[1]), message[2], seq_map[message[3]])

    def map_node(node):
        if is_leaf(node):
            return ('L', node[1], node[2], tuple((key, map_version(key, version)) for key, version in node[3]))
        return ('I', node[1], node[2], tuple(map_message(message) for message in node[3]),
                tuple(map_node(child) for child in node[4]))

    tree, frontend, counter, snap_tree, snap_counter, truth, snap_truth, applied = state
    return (map_node(tree), tuple(map_message(entry) for entry in frontend), seq_map[counter],
            map_node(snap_tree), seq_map[snap_counter],
            tuple((map_version(key, pair[0]), pair[1]) for key, pair in enumerate(truth)),
            tuple((map_version(key, pair[0]), pair[1]) for key, pair in enumerate(snap_truth)),
            tuple((key, None if value is None else map_version(key, value)) for key, value in applied))


# ---------------------------------------------------------------- 写
def op_front_end_write(arm, state, key, is_delete):
    tree, frontend, counter, snap_tree, snap_counter, truth, snap_truth, applied = state
    version = next_version(state, key)
    seq = 0
    if arm['kind'] == 'yiG':                           # T3：写进前端时取号
        counter += 1
        seq = counter
    frontend = tuple(entry for entry in frontend if entry[0] != key) + ((key, version, is_delete, seq),)
    return (tree, frontend, counter, snap_tree, snap_counter, set_truth(truth, key, version, is_delete), snap_truth, applied)


def op_direct_write(arm, state, key, is_delete):
    """甲：不走前端的写进根缓冲，并把前端里同 key 的条目逐出。"""
    tree, frontend, counter, snap_tree, snap_counter, truth, snap_truth, applied = state
    version = next_version(state, key)
    message = (key, version, is_delete, 0)
    frontend = tuple(entry for entry in frontend if entry[0] != key)
    if is_leaf(tree):
        tree = apply_to_leaf(arm, tree, [message])
    else:
        buffer = insert_messages(arm, tree[3], [message])
        if len(buffer) > BUFFER_CAPACITY:
            return None
        tree = with_buffer(tree, buffer)
    return (tree, frontend, counter, snap_tree, snap_counter, set_truth(truth, key, version, is_delete), snap_truth, applied)


def op_second_writer(arm, state, key):
    """变异：丙的前端树上多一个直落叶的写者（不进前端、不逐出前端里的同 key 条目）。"""
    tree, frontend, counter, snap_tree, snap_counter, truth, snap_truth, applied = state
    version = next_version(state, key)
    path = leaf_path_for_key(tree, key)
    tree = replace_at(tree, path, apply_to_leaf(arm, node_at(tree, path), [(key, version, False, 0)]))
    return (tree, frontend, counter, snap_tree, snap_counter, set_truth(truth, key, version, False), snap_truth, applied)


# ---------------------------------------------------------------- flush 与下推
TIE = 'TIE'


def purge_key_on_path(tree, key):
    """甲-R4：flush 时清掉路径上（按当前 pivot）比它旧的同 key 消息。前端里的条目总比缓冲里的新（不走前端的写逐出前端）。"""
    if is_leaf(tree):
        return tree
    index = child_index_for_key(tree, key)
    children = list(tree[4])
    children[index] = purge_key_on_path(children[index], key)
    return make_inner(tree[1], tree[2], [message for message in tree[3] if message[0] != key], children)


def flush_entry(arm, tree, entry, respect_capacity):
    """一条前端条目进树：丙 / 甲 直落叶，乙 进根缓冲。返回新树、None（放不下）或 TIE。"""
    if arm['kind'] in ('bing', 'jia') or is_leaf(tree):
        if arm['kind'] == 'jia':
            tree = purge_key_on_path(tree, entry[0])
        path = leaf_path_for_key(tree, entry[0])
        leaf = apply_to_leaf(arm, node_at(tree, path), [entry])
        return TIE if leaf is None else replace_at(tree, path, leaf)
    buffer = insert_messages(arm, tree[3], [entry])
    if respect_capacity and len(buffer) > BUFFER_CAPACITY:
        return None
    return with_buffer(tree, buffer)


def op_flush_one(arm, state):
    tree, frontend, counter, snap_tree, snap_counter, truth, snap_truth, applied = state
    if not frontend:
        return None
    entry = frontend[0]
    tree = flush_entry(arm, tree, entry, True)
    if tree is None or tree is TIE:
        return tree
    if arm['kind'] == 'bing':
        applied = applied + ((entry[0], visible(entry)),)
    return (tree, frontend[1:], counter, snap_tree, snap_counter, truth, snap_truth, applied)


def push_node(arm, node, child_index, respect_capacity):
    """把 node 缓冲里归第 child_index 个孩子管的消息推下去（部分下推）。返回新 node、None 或 TIE。"""
    child = node[4][child_index]
    moving = [message for message in node[3] if child[1] <= message[0] < child[2]]
    if not moving:
        return None
    staying = [message for message in node[3] if not (child[1] <= message[0] < child[2])]
    if is_leaf(child):
        new_child = apply_to_leaf(arm, child, moving)
        if new_child is None:
            return TIE
    else:
        buffer = insert_messages(arm, child[3], moving)
        if respect_capacity and len(buffer) > BUFFER_CAPACITY:
            return None
        new_child = with_buffer(child, buffer)
    children = list(node[4])
    children[child_index] = new_child
    return make_inner(node[1], node[2], staying, children)


def op_push(arm, state, path, child_index):
    tree = state[0]
    new_node = push_node(arm, node_at(tree, path), child_index, True)
    if new_node is None or new_node is TIE:
        return new_node
    return (replace_at(tree, path, new_node),) + state[1:]


def cascade(arm, tree):
    """发布时的确定性级联：最浅的超容量缓冲把消息最多的那个孩子整份推下去，直到都不超。"""
    while True:
        over = [(len(path), path) for path, node in all_paths(tree)
                if not is_leaf(node) and len(node[3]) > BUFFER_CAPACITY]
        if not over:
            return tree
        path = min(over)[1]
        node = node_at(tree, path)
        counts = [sum(1 for message in node[3] if child[1] <= message[0] < child[2]) for child in node[4]]
        index = max(range(len(counts)), key=lambda position: (counts[position], -position))
        new_node = push_node(arm, node, index, False)
        if new_node is TIE:
            return TIE
        tree = replace_at(tree, path, new_node)


def drain_all(arm, tree):
    """D11 已定项 5 的排空：自上而下把每层缓冲整份推到底。"""
    if is_leaf(tree):
        return tree
    node = tree
    for index in range(len(node[4])):
        pushed = push_node(arm, node, index, False)
        if pushed is TIE:
            return TIE
        if pushed is not None:
            node = pushed
    children = []
    for child in node[4]:
        drained = drain_all(arm, child)
        if drained is TIE:
            return TIE
        children.append(drained)
    return make_inner(node[1], node[2], node[3], children)


# ---------------------------------------------------------------- 重组
def insert_sibling_pair(tree, path, left, right):
    """把 path 上那个节点换成 left、right 两个兄弟；path 是根就长高一层。父节点满了 ⇒ None。"""
    if not path:
        if tree_height(tree) >= MAX_HEIGHT:
            return None
        return make_inner(left[1], right[2], (), (left, right))
    parent = node_at(tree, path[:-1])
    if len(parent[4]) >= MAX_FANOUT:
        return None
    children = list(parent[4])
    children[path[-1]:path[-1] + 1] = [left, right]
    return replace_at(tree, path[:-1], with_children(parent, children))


def op_split_leaf(arm, state, path, position):
    tree = state[0]
    leaf = node_at(tree, path)
    if not leaf[1] < position < leaf[2]:
        return None
    left = make_leaf(leaf[1], position, [item for item in leaf[3] if item[0] < position])
    right = make_leaf(position, leaf[2], [item for item in leaf[3] if item[0] >= position])
    tree = insert_sibling_pair(tree, path, left, right)
    return None if tree is None else (tree,) + state[1:]


def op_split_inner(arm, state, path):
    tree = state[0]
    node = node_at(tree, path)
    if len(node[4]) != 2:
        return None
    first, second = node[4]
    if arm['split'] == 'pivot':                     # 按 pivot 区间各带走自己那一半缓冲
        left_buffer = [message for message in node[3] if message[0] < first[2]]
        right_buffer = [message for message in node[3] if message[0] >= first[2]]
    else:                                           # 变异 split_keep：左半留下全部消息
        left_buffer, right_buffer = list(node[3]), []
    tree = insert_sibling_pair(tree, path, make_inner(first[1], first[2], left_buffer, (first,)),
                               make_inner(second[1], second[2], right_buffer, (second,)))
    return None if tree is None else (tree,) + state[1:]


def op_merge(arm, state, parent_path, index):
    tree = state[0]
    parent = node_at(tree, parent_path)
    if index + 1 >= len(parent[4]):
        return None
    first, second = parent[4][index], parent[4][index + 1]
    if is_leaf(first):
        merged = make_leaf(first[1], second[2], first[3] + second[3])
    else:
        if len(first[4]) + len(second[4]) > MAX_FANOUT:
            return None
        buffer = insert_messages(arm, first[3], second[3])
        if len(buffer) > BUFFER_CAPACITY:
            return None
        merged = make_inner(first[1], second[2], buffer, first[4] + second[4])
    children = parent[4][:index] + (merged,) + parent[4][index + 2:]
    return (replace_at(tree, parent_path, with_children(parent, children)),) + state[1:]


def op_collapse(arm, state):
    """根只剩一个孩子 ⇒ 树变矮。T1：先把根缓冲按下推规则推进孩子，孩子当新根；fold_up：留下根，孩子缓冲当新来的并进根缓冲。"""
    tree = state[0]
    if is_leaf(tree) or len(tree[4]) != 1:
        return None
    child = tree[4][0]
    if arm['collapse'] == 'fold_up' and not is_leaf(child):
        buffer = insert_messages(arm, tree[3], child[3])
        if len(buffer) > BUFFER_CAPACITY:
            return None
        return (make_inner(tree[1], tree[2], buffer, child[4]),) + state[1:]
    if is_leaf(child):
        new_root = apply_to_leaf(arm, child, list(tree[3]))
        if new_root is None:
            return TIE
    else:
        buffer = insert_messages(arm, child[3], tree[3])
        if len(buffer) > BUFFER_CAPACITY:
            return None
        new_root = with_buffer(child, buffer)
    return (new_root,) + state[1:]


def op_rebalance(arm, state, parent_path, index, direction):
    tree = state[0]
    parent = node_at(tree, parent_path)
    if index + 1 >= len(parent[4]):
        return None
    left, right = parent[4][index], parent[4][index + 1]
    if is_leaf(left):                               # 叶：挪一个 key 位，父缓冲罩着两边、不动
        if direction == 'to_right' and left[2] - left[1] >= 2:
            boundary = left[2] - 1
        elif direction == 'to_left' and right[2] - right[1] >= 2:
            boundary = right[1] + 1
        else:
            return None
        items = dict(left[3])
        items.update(dict(right[3]))
        new_left = make_leaf(left[1], boundary, [item for item in items.items() if item[0] < boundary])
        new_right = make_leaf(boundary, right[2], [item for item in items.items() if item[0] >= boundary])
    else:                                           # 内部节点：挪一个孩子，T2 管它缓冲里的消息怎么搬
        if direction == 'to_right' and len(left[4]) >= 2 and len(right[4]) < MAX_FANOUT:
            source, target, moved = left, right, left[4][-1]
        elif direction == 'to_left' and len(right[4]) >= 2 and len(left[4]) < MAX_FANOUT:
            source, target, moved = right, left, right[4][0]
        else:
            return None
        in_range = [moved[1] <= message[0] < moved[2] for message in source[3]]
        if arm['rebalance'] == 'pivot':             # 按 pivot 区间搬
            moving_flags = in_range
        else:                                       # 最弱读法 present：只搬挪走的那棵子树里「有这个 key」的消息
            present = keys_mentioned(moved)
            moving_flags = [flag and message[0] in present for flag, message in zip(in_range, source[3])]
        moving = [message for flag, message in zip(moving_flags, source[3]) if flag]
        staying = [message for flag, message in zip(moving_flags, source[3]) if not flag]
        target_buffer = insert_messages(arm, target[3], moving)
        if len(target_buffer) > BUFFER_CAPACITY:
            return None
        source_children = tuple(child for child in source[4] if child is not moved)
        if direction == 'to_right':
            new_left = make_inner(left[1], moved[1], staying, source_children)
            new_right = make_inner(moved[1], right[2], target_buffer, (moved,) + right[4])
        else:
            new_left = make_inner(left[1], moved[2], target_buffer, left[4] + (moved,))
            new_right = make_inner(moved[2], right[2], staying, source_children)
    children = list(parent[4])
    children[index:index + 2] = [new_left, new_right]
    return (replace_at(tree, parent_path, with_children(parent, children)),) + state[1:]


def op_rebuild(arm, state):
    """D11 已定项 5：索引重建之前各层缓冲先排空；重建成一片叶（改 ε / 改节点大小的形状变化）。变异 nodrain：只拿叶。"""
    tree = state[0]
    if is_leaf(tree):
        return None
    if arm['rebuild'] == 'drain':
        tree = drain_all(arm, tree)
        if tree is TIE:
            return TIE
    return (make_leaf(0, KEY_COUNT, leaf_items(tree).items()),) + state[1:]


# ---------------------------------------------------------------- 发布与崩溃
def g3_verdicts(old_tree, new_tree, applied):
    """G3 的两种写法。逻辑：叶上 key → 值 变了的，每一处都要对得上这次排空的前端条目。
    物理：新出现的叶节点（区间或内容变了）要有一条这次排空的条目落在它的区间里——分裂、再平衡会让它红（误报）。"""
    old_items, new_items = leaf_items(old_tree), leaf_items(new_tree)
    last_applied = dict(applied)
    logical_red = any(old_items.get(key) != new_items.get(key)
                      and (key not in last_applied or last_applied[key] != new_items.get(key))
                      for key in range(KEY_COUNT))
    old_leaves = {node for _, node in all_paths(old_tree) if is_leaf(node)}
    physical_red = any(is_leaf(node) and node not in old_leaves
                       and not any(node[1] <= key < node[2] for key in last_applied)
                       for _, node in all_paths(new_tree))
    return logical_red, physical_red


def op_publish(arm, state):
    """按臂把前端排空（丙 / 甲到叶，乙到根缓冲后确定性级联），整棵树落盘；计数器随发布落盘（T3）。"""
    tree, frontend, counter, snap_tree, snap_counter, truth, snap_truth, applied = state
    publish_drain = tuple((entry[0], visible(entry)) for entry in frontend)
    for entry in frontend:
        tree = flush_entry(arm, tree, entry, False)
        if tree is TIE:
            return TIE, ()
        if arm['kind'] == 'bing':
            applied = applied + ((entry[0], visible(entry)),)
    if arm['kind'] in ('yiD', 'yiG') and not is_leaf(tree):
        tree = cascade(arm, tree)
        if tree is TIE:
            return TIE, ()
    flags = ()
    if arm['kind'] == 'bing':
        logical_red, physical_red = g3_verdicts(snap_tree, tree, applied)
        flags = (('G3',) if logical_red else ()) + (('G3_physical',) if physical_red else ())
        if g3_verdicts(snap_tree, tree, publish_drain)[0]:          # 字面「这次排空」：只认发布时排空的，不认窗口中途 flush 的
            flags += ('G3_publish_only',)
    return (tree, (), counter, tree, counter, truth, truth, ()), flags


def op_crash(arm, state):
    snap_tree, snap_counter, snap_truth = state[3], state[4], state[6]
    counter = snap_counter if arm['counter_on_crash'] == 'persisted' else 0
    return (snap_tree, (), counter, snap_tree, snap_counter, snap_truth, snap_truth, ())


# ---------------------------------------------------------------- 枚举
def successors(arm, state, with_publish):
    """产出 (标签, 种类, 新状态或 TIE, 附带的检查旗标)。"""
    tree = state[0]
    for key in range(KEY_COUNT):
        for is_delete in (False, True):
            verb = 'del' if is_delete else 'put'
            if not arm.get('direct_only'):
                yield 'fe_%s_%s' % (verb, KEY_NAMES[key]), 'write', op_front_end_write(arm, state, key, is_delete), ()
            if arm['kind'] == 'jia':
                yield 'direct_%s_%s' % (verb, KEY_NAMES[key]), 'write', op_direct_write(arm, state, key, is_delete), ()
        if arm.get('second_writer'):
            yield 'leaf_put_%s' % KEY_NAMES[key], 'write', op_second_writer(arm, state, key), ()
    yield 'flush', 'flush', op_flush_one(arm, state), ()
    for path, node in all_paths(tree):
        label_path = ''.join(str(index) for index in path) or 'r'
        if is_leaf(node):
            for position in range(node[1] + 1, node[2]):
                yield 'split_leaf@%s/%d' % (label_path, position), 'split_leaf', op_split_leaf(arm, state, path, position), ()
            continue
        for index in range(len(node[4])):
            yield 'push@%s>%d' % (label_path, index), 'push', op_push(arm, state, path, index), ()
        yield 'split_inner@%s' % label_path, 'split_inner', op_split_inner(arm, state, path), ()
        for index in range(len(node[4]) - 1):
            yield 'merge@%s:%d' % (label_path, index), 'merge', op_merge(arm, state, path, index), ()
            for direction in ('to_right', 'to_left'):
                yield ('rebalance@%s:%d:%s' % (label_path, index, direction), 'rebalance',
                       op_rebalance(arm, state, path, index, direction), ())
    yield 'collapse', 'collapse', op_collapse(arm, state), ()
    yield 'rebuild', 'rebuild', op_rebuild(arm, state), ()
    if with_publish:
        published, flags = op_publish(arm, state)
        yield 'publish', 'publish', published, flags
        yield 'crash', 'crash', op_crash(arm, state), ()


def state_violations(arm, state):
    found = []
    truth = state[5]
    for key in range(KEY_COUNT):
        values = read_key(arm, state, key)
        if len(values) > 1:
            found.append('undefined')
        elif truth_visible(truth[key]) not in values:
            found.append('stale')
    for _, node in all_paths(state[0]):
        if is_leaf(node):
            continue
        if any(not node[1] <= message[0] < node[2] for message in node[3]):
            found.append('G2')
        if arm['kind'] == 'yiD' and len({message[0] for message in node[3]}) != len(node[3]):
            found.append('yiD_unique')
    return found


def transition_violations(arm, kind, before, after, flags):
    found = list(flags)
    if kind in STRUCTURAL and all_reads(arm, before) != all_reads(arm, after):
        found.append('G1')
    if kind == 'crash' and any(truth_visible(after[5][key]) not in read_key(arm, after, key) for key in range(KEY_COUNT)):
        found.append('u2_recovered')
    return found


def breadth_first_search(arm, start, depth_limit, with_publish):
    start = relabel_state(start)
    parents = {start: None}
    frontier, first_hits = [start], {}

    def path_to(state):
        labels = []
        while parents[state] is not None:
            state, label = parents[state]
            labels.append(label)
        return labels[::-1]

    for _ in range(depth_limit):
        next_frontier = []
        for state in frontier:
            for label, kind, successor, flags in successors(arm, state, with_publish):
                if successor is None:
                    continue
                if successor is TIE:
                    first_hits.setdefault('undefined', path_to(state) + [label])
                    continue
                found = transition_violations(arm, kind, state, successor, flags)
                successor = relabel_state(successor)
                found += state_violations(arm, successor)
                for category in found:
                    first_hits.setdefault(category, path_to(state) + [label])
                if successor not in parents:
                    parents[successor] = (state, label)
                    next_frontier.append(successor)
        frontier = next_frontier
        if not frontier:
            break
    return first_hits, len(parents)


def random_walks(arm, start, walk_count, walk_length, seed, with_publish):
    """固定种子的随机游走：每步在当前可做的操作里均匀挑一个；记每类命中最短的那段前缀。"""
    generator = random.Random(seed)
    first_hits, steps = {}, 0
    for _ in range(walk_count):
        state, labels = relabel_state(start), []
        for _ in range(walk_length):
            options = [option for option in successors(arm, state, with_publish) if option[2] is not None]
            label, kind, successor, flags = generator.choice(options)
            labels.append(label)
            steps += 1
            found = ['undefined'] if successor is TIE else transition_violations(arm, kind, state, successor, flags)
            if successor is not TIE:
                successor = relabel_state(successor)
                found += state_violations(arm, successor)
            for category in found:
                if category not in first_hits or len(labels) < len(first_hits[category]):
                    first_hits[category] = list(labels)
            if successor is TIE:
                break
            state = successor
    return first_hits, steps


# ---------------------------------------------------------------- 起点、臂、主程序
def populated(node):
    if is_leaf(node):
        return make_leaf(node[1], node[2], [(key, 1) for key in range(node[1], node[2])])
    return make_inner(node[1], node[2], node[3], [populated(child) for child in node[4]])


def start_state(tree, is_populated):
    if is_populated:
        tree = populated(tree)
    truth = tuple(((1, False) if is_populated else (0, True)) for _ in range(KEY_COUNT))
    return (tree, (), 0, tree, 0, truth, truth, ())


def leaf(low, high):
    return make_leaf(low, high, ())


def inner(low, high, *children):
    return make_inner(low, high, (), children)


START_SHAPES = [
    ('h1', leaf(0, 3)),
    ('h2', inner(0, 3, leaf(0, 1), leaf(1, 3))),
    ('h3_rebalance', inner(0, 3, inner(0, 2, leaf(0, 1), leaf(1, 2)), inner(2, 3, leaf(2, 3)))),
    ('h3_collapse', inner(0, 3, inner(0, 3, leaf(0, 1), leaf(1, 3)))),
    ('h4_chain', inner(0, 3, inner(0, 3, inner(0, 3, leaf(0, 1), leaf(1, 3))))),
    ('h4_rebalance', inner(0, 3, inner(0, 2, inner(0, 1, leaf(0, 1)), inner(1, 2, leaf(1, 2))),
                           inner(2, 3, inner(2, 3, leaf(2, 3))))),
]

BASE_ARM = dict(collapse='t1', rebalance='pivot', split='pivot', rebuild='drain', counter_on_crash='persisted',
                second_writer=False)


def make_arm(kind, **overrides):
    arm = dict(BASE_ARM, kind=kind)
    arm.update(overrides)
    return arm


ARMS = [
    ('bing_T6', make_arm('bing')),
    ('jia_R4_T', make_arm('jia')),
    ('yi_D_T', make_arm('yiD')),
    ('yi_G_T', make_arm('yiG', collapse='fold_up')),          # 乙-G-T 只加了 T2，塌缩按最弱读法 fold_up
    ('yi_G_T+T1', make_arm('yiG')),
    ('jia_R4_T2present', make_arm('jia', rebalance='present')),   # T2「随 key 搬走」的最弱读法
    ('yi_D_T2present', make_arm('yiD', rebalance='present')),
    ('yi_G_T2present', make_arm('yiG', collapse='fold_up', rebalance='present')),
    ('jia_R4_foldup', make_arm('jia', collapse='fold_up')),       # 阳性对照：第二轮打中的塌缩最弱读法
    ('yi_D_foldup', make_arm('yiD', collapse='fold_up')),
    # 不走前端的树（extent / dirent，按树轴四条臂都留缓冲）：只有直进根缓冲的写，缓冲按位置定新旧
    ('extent_tree_T1T2', make_arm('jia', direct_only=True)),
    ('extent_tree_T2present', make_arm('jia', direct_only=True, rebalance='present')),
    ('extent_tree_foldup', make_arm('jia', direct_only=True, collapse='fold_up')),
]

MUTATIONS = [
    ('M1_jia_split_keep', make_arm('jia', split='keep')),
    ('M2_yiD_split_keep', make_arm('yiD', split='keep')),
    ('M3_yiG_split_keep', make_arm('yiG', collapse='fold_up', split='keep')),
    ('M4_jia_rebuild_nodrain', make_arm('jia', rebuild='nodrain')),
    ('M5_yiD_rebuild_nodrain', make_arm('yiD', rebuild='nodrain')),
    ('M6_yiG_rebuild_nodrain', make_arm('yiG', collapse='fold_up', rebuild='nodrain')),
    ('M7_bing_second_writer', make_arm('bing', second_writer=True)),
    ('M8_yiG_counter_reset', make_arm('yiG', collapse='fold_up', counter_on_crash='reset')),
]

HIT_ORDER = ('stale', 'undefined', 'u2_recovered', 'G1', 'G2', 'G3', 'G3_physical', 'G3_publish_only', 'yiD_unique')


def format_hits(first_hits):
    parts = ['%s=%d:%s' % (category, len(first_hits[category]), ','.join(first_hits[category]))
             for category in HIT_ORDER if category in first_hits]
    return ' '.join(parts) if parts else 'none'


def select_arms(selector):
    table = dict(ARMS + MUTATIONS)
    if selector == 'arms':
        return ARMS
    if selector == 'mutations':
        return MUTATIONS
    return [(name, table[name]) for name in selector.split(',')]


def main():
    """用法：c161_r3_tree.py bfs <臂|arms|mutations> <深度> <带不带发布 0/1>
             c161_r3_tree.py walk <臂|arms|mutations> <步长> <带不带发布 0/1> <游走次数>"""
    import zlib
    mode, selector, depth, with_publish = sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4] == '1'
    walk_count = int(sys.argv[5]) if mode == 'walk' else 0
    emitted = 0
    for name, arm in select_arms(selector):
        for shape_name, shape in START_SHAPES:
            for is_populated in (False, True):
                start = start_state(shape, is_populated)
                label = '%s%s' % (shape_name, '+full' if is_populated else '')
                if mode == 'bfs':
                    hits, states = breadth_first_search(arm, start, depth, with_publish)
                    line = 'part=A mode=bfs arm=%s start=%s publish=%d depth=%d states=%d %s' % (
                        name, label, with_publish, depth, states, format_hits(hits))
                else:
                    seed = zlib.crc32(('%s|%s|%d' % (name, label, with_publish)).encode())
                    hits, steps = random_walks(arm, start, walk_count, depth, seed, with_publish)
                    line = 'part=A mode=walk arm=%s start=%s publish=%d walks=%d length=%d seed=%d steps=%d %s' % (
                        name, label, with_publish, walk_count, depth, seed, steps, format_hits(hits))
                print(line, flush=True)
                emitted += 1
    print('emitted=%d' % emitted)


if __name__ == '__main__':
    main()
