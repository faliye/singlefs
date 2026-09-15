#!/usr/bin/env python3
"""C161（缓冲两级并存而衔接没定） 第一轮攻方腿（Opus）的小模型，只用 Python 标准库。

树的形状写死成高度 3：根 R（缓冲 root_buffer）→ 孩子 C（缓冲 child_buffer）→ 叶 L，全部 key 走同一条路径，
够摆出「同一个 key 在前端、根缓冲、孩子缓冲、叶里各有一条」的全部组合。前端 FE 在内存里：活跃批次加若干冻结批次。

一条条目是 Entry(key, kind, value, seq, arrival, publish)：
  kind     'P' 写入，'D' 墓碑
  value    每次写入取一个新值（值相等即同一次写入）
  seq      盘上的序号字段，口径由臂的 seq_mode 定
  arrival  内存里的到达序（时间序的真值）；从盘上重读之后丢失，记 OLD
  publish  进前端或进树时的发布号；从盘上重读之后丢失，记 OLD
判据：U1 读出的值 != 按时间序应得的值；U2 崩溃恢复出的内容 != 发布时刻的真值（另查 I-3.1 与反向索引两条账）。
复跑：python3 c161_model.py > c161-model-output.txt（产物末行 emitted=N，行数对不上即作废）。
"""
import itertools
import random
import sys
from collections import namedtuple

Entry = namedtuple('Entry', 'key kind value seq arrival publish')
OLD = -1
ACCOUNTING_STATISTICS = ('s', 'alloc')   # 带「代」的记账 key：(统计量, 代)
GENERIC_KEY = 'k'                        # 不带「代」的 key（反向索引 / LRU 的 key 布局仓里没定，按不带代算）


def entry_outcome(entry):
    """一条条目读出来的值：墓碑与不存在都读成 None。"""
    if entry is None:
        return None
    return entry.value if entry.kind == 'P' else None


def new_state():
    return {
        'front_end': {}, 'frozen_batches': [], 'root_buffer': [], 'child_buffer': [], 'leaf': {},
        'durable': ((), (), ()), 'truth': {}, 'truth_at_publish': {},
        'current_publish': 0, 'seq_count_in_publish': {}, 'global_seq': 0,
        'arrival_counter': 0, 'fresh_value_counter': 0,
        'recovered_content_wrong': False, 'tie_broken_adversarially': False,
        'extent_counter': 0, 'extents': {}, 'extents_durable': {}, 'publish_boundary': None,
        'crash_labels': frozenset(),
    }


def copy_state(state):
    copied = dict(state)
    for field in ('front_end', 'leaf', 'truth', 'truth_at_publish', 'seq_count_in_publish',
                  'extents', 'extents_durable'):
        copied[field] = dict(state[field])
    copied['frozen_batches'] = [dict(batch) for batch in state['frozen_batches']]
    copied['root_buffer'] = list(state['root_buffer'])
    copied['child_buffer'] = list(state['child_buffer'])
    return copied


def next_seq(state, arm, key):
    """盘上 seq 的三种口径。"""
    mode = arm['seq_mode']
    if mode == 'per_publish':      # D8 已定项 10：本次发布里被写进树的第几次，从 1 起
        count = state['seq_count_in_publish'].get(key, 0) + 1
        state['seq_count_in_publish'][key] = count
        return count
    state['global_seq'] += 1
    if mode == 'global':           # 反向接受条款里的「全池单调」
        return state['global_seq']
    if mode == 'adversarial':      # 越新越小：拿来证明一条臂根本不读 seq
        return -state['global_seq']
    raise ValueError(mode)


def record_truth(state, key, kind, value=None):
    """按时间序记真值；返回这次写入的值与到达序。"""
    state['arrival_counter'] += 1
    if kind == 'P':
        if value is None:
            state['fresh_value_counter'] += 1
            value = state['fresh_value_counter']
        state['truth'][key] = value
    else:
        value = None
        state['truth'].pop(key, None)
    return value, state['arrival_counter']


def front_end_update(state, key, kind, value=None, tag=None):
    """写进前端：前端在内存里按到达序去重（后到的替换先到的），不靠 seq。"""
    value, arrival = record_truth(state, key, kind, value)
    publish_tag = state['current_publish'] if tag is None else tag
    state['front_end'][key] = Entry(key, kind, value, None, arrival, publish_tag)
    return [state]


def direct_update(state, arm, key, kind):
    """不走前端的写。丙：前端树上没有这类写者，一律进前端。"""
    target = arm['direct_target']
    if target == 'front_end':
        return front_end_update(state, key, kind)
    value, arrival = record_truth(state, key, kind)
    entry = Entry(key, kind, value, next_seq(state, arm, key), arrival, state['current_publish'])
    if arm.get('direct_evicts_front_end'):
        # 甲-R4 的收严：不走前端的写比前端里同 key 的条目新，顺手把它们逐出前端，免得读前端先读到旧的
        state['front_end'].pop(key, None)
        for batch in state['frozen_batches']:
            batch.pop(key, None)
        state['frozen_batches'] = [batch for batch in state['frozen_batches'] if batch]
    if target == 'leaf_blind':
        state['leaf'][key] = entry
        return [state]
    if target == 'root_buffer':
        buffer_insert(arm, state, 'root_buffer', entry)
        return [state]
    raise ValueError(target)


def arrival_older(state, candidate_older, candidate_newer):
    """只用内存里的信息判 candidate_older 是不是更旧。

    还没发布的（到达序在、发布号不小于当前窗口）一律比已发布的新；两条都已发布（或都从盘上重读）时判不出，返回 None。
    """
    current = state['current_publish']

    def is_unpublished(entry):
        return entry.arrival != OLD and entry.publish >= current

    older_is_unpublished = is_unpublished(candidate_older)
    newer_is_unpublished = is_unpublished(candidate_newer)
    if older_is_unpublished and newer_is_unpublished:
        return candidate_older.arrival < candidate_newer.arrival
    if newer_is_unpublished:
        return True
    if older_is_unpublished:
        return False
    return None


def buffer_insert(arm, state, buffer_name, entry):
    """把一条消息放进某一层缓冲。位置即新旧：放进来的一定比这一层已有的同 key 消息新（来自前端或上一层）。"""
    policy = arm['buffer_policy']
    mutations = arm['mutations']
    buffer = state[buffer_name]
    has_same_key = any(message.key == entry.key for message in buffer)
    if policy in ('arrival', 'seq'):
        if 'push_inserts_before' in mutations and buffer_name == 'child_buffer':
            buffer.insert(0, entry)      # 变异：下推来的消息排到孩子已有消息的前面
        else:
            buffer.append(entry)
        return
    if policy == 'dedup':
        if has_same_key and 'dedup_keeps_old' in mutations:
            return                        # 变异：缓冲区里已有同 key 就丢掉新来的
        if has_same_key and 'push_keeps_child' in mutations and buffer_name == 'child_buffer':
            return                        # 变异：下推时孩子已有的同 key 消息不被替换
        state[buffer_name] = [message for message in buffer if message.key != entry.key] + [entry]
        return
    if policy == 'dedup_arrival_guard':
        for message in buffer:
            if message.key == entry.key and arrival_older(state, entry, message):
                return                    # 进来的比已有的旧（乱序 flush）：不替换
        state[buffer_name] = [message for message in buffer if message.key != entry.key] + [entry]
        return
    raise ValueError(policy)


def leaf_apply_one(state, arm, entry, rule):
    """一条条目落进叶。返回后继状态的列表（盘上次序没有定义的平局，两边都走）。"""
    existing = state['leaf'].get(entry.key)
    if existing is None or rule == 'blind':
        state['leaf'][entry.key] = entry
        return [state]
    if rule == 'seq_guard':
        if entry.seq > existing.seq:
            state['leaf'][entry.key] = entry
            return [state]
        if entry.seq < existing.seq or entry_outcome(entry) == entry_outcome(existing):
            return [state]
        replaced = copy_state(state)
        replaced['leaf'][entry.key] = entry
        replaced['tie_broken_adversarially'] = True
        state['tie_broken_adversarially'] = True
        return [state, replaced]
    if rule == 'arrival_guard':
        if arrival_older(state, existing, entry) is False:
            return [state]                 # 叶里那条更新（同一窗口里后到）：不覆盖
        state['leaf'][entry.key] = entry   # 更旧，或两条都已发布（按不变量，缓冲区里的新）
        return [state]
    raise ValueError(rule)


def flush_entry(state, arm, front_entry):
    """前端的一条进树：此刻按口径取 seq（D8 已定项 10：被写进树的第几次）。"""
    entry = front_entry._replace(seq=next_seq(state, arm, front_entry.key))
    target = arm['flush_target']
    if target == 'root_buffer':
        buffer_insert(arm, state, 'root_buffer', entry)
        return [state]
    if target == 'leaf_blind':
        state['leaf'][entry.key] = entry
        return [state]
    if target == 'leaf_seq_guard':
        return leaf_apply_one(state, arm, entry, 'seq_guard')
    if target == 'leaf_arrival_guard':
        return leaf_apply_one(state, arm, entry, 'arrival_guard')
    if target == 'leaf_purge':
        if 'purge_skipped' not in arm['mutations']:
            for name in ('root_buffer', 'child_buffer'):
                state[name] = [message for message in state[name]
                               if not (message.key == entry.key and arrival_older(state, message, entry))]
        rule = 'blind' if 'purge_leaf_blind' in arm['mutations'] else 'arrival_guard'
        return leaf_apply_one(state, arm, entry, rule)
    raise ValueError(target)


def flush_entries(states, arm, entries):
    for entry in entries:
        states = [successor for state in states for successor in flush_entry(state, arm, entry)]
    return states


def drain_front_end(state, arm, tag_limit=None):
    """冻结批次从旧到新、再是活跃批次，全部进树；批次内按 key 排序（排序丢掉时间序）。

    tag_limit 给了就只排空标签不大于它的（发布集边界：前端按窗口打标签）。
    """
    to_flush = []
    kept_batches = []
    for batch in state['frozen_batches'] + [state['front_end']]:
        kept = {}
        for key in sorted(batch, key=repr):
            entry = batch[key]
            if tag_limit is None or entry.publish <= tag_limit:
                to_flush.append(entry)
            else:
                kept[key] = entry
        kept_batches.append(kept)
    state['frozen_batches'] = [batch for batch in kept_batches[:-1] if batch]
    state['front_end'] = kept_batches[-1]
    return flush_entries([state], arm, to_flush)


def has_front_end_content(state):
    return bool(state['front_end']) or any(state['frozen_batches'])


def op_flush(state, arm):
    return drain_front_end(state, arm) if has_front_end_content(state) else []


def op_freeze(state, arm):
    if not state['front_end']:
        return []
    state['frozen_batches'].append(state['front_end'])
    state['front_end'] = {}
    return [state]


def op_flush_batch(state, arm, index):
    """并行 flush 的形状：flush 某一个冻结批次；index = -1 且有两个以上批次时就是乱序。"""
    if not state['frozen_batches']:
        return []
    batch = state['frozen_batches'].pop(index)
    return flush_entries([state], arm, [batch[key] for key in sorted(batch, key=repr)])


def seq_orderings(messages):
    """seq 口径的缓冲：同 key 同 seq 而值不同的消息谁先谁后，盘上没有定义——每种排列都走一遍。"""
    groups = {}
    for message in messages:
        groups.setdefault((repr(message.key), message.seq), []).append(message)
    per_group = []
    tie_with_different_outcomes = False
    for group_key in sorted(groups):
        group = groups[group_key]
        if len({entry_outcome(message) for message in group}) > 1:
            tie_with_different_outcomes = True
            per_group.append(list(itertools.permutations(group)))
        else:
            per_group.append([tuple(group)])
    orderings = [[message for group in combination for message in group]
                 for combination in itertools.product(*per_group)]
    return orderings, tie_with_different_outcomes


def apply_messages_to_leaf(state, arm, messages):
    if arm['buffer_policy'] == 'seq':
        orderings, tie = seq_orderings(messages)
    else:
        orderings, tie = [list(messages)], False
    branches = [state] + [copy_state(state) for _ in orderings[1:]]
    results = []
    for branch, ordering in zip(branches, orderings):
        if tie:
            branch['tie_broken_adversarially'] = True
        states = [branch]
        for message in ordering:
            states = [successor for current in states
                      for successor in leaf_apply_one(current, arm, message, arm['leaf_apply'])]
        results.extend(states)
    return results


def op_push_root(state, arm):
    if arm['buffer_policy'] is None or not state['root_buffer']:
        return []
    messages = state['root_buffer']
    state['root_buffer'] = []
    for message in messages:
        buffer_insert(arm, state, 'child_buffer', message)
    return [state]


def op_push_child(state, arm):
    if arm['buffer_policy'] is None or not state['child_buffer']:
        return []
    messages = state['child_buffer']
    state['child_buffer'] = []
    return apply_messages_to_leaf(state, arm, messages)


def newest_in_buffer(arm, buffer, key):
    same_key = [message for message in buffer if message.key == key]
    if not same_key:
        return None
    if arm['buffer_policy'] == 'seq':    # 同一层里谁新由 seq 定（D5 已定项 3 第 2 行「后者由 seq 定」）
        top = max(message.seq for message in same_key)
        return {entry_outcome(message) for message in same_key if message.seq == top}
    return {entry_outcome(same_key[-1])}  # 位置即新旧：同一层里后放进来的新


def read_tree(state, arm, key):
    """不看前端，只读树（崩溃恢复之后的读法也是它）。返回可能读出的值的集合。"""
    leaf_entry = state['leaf'].get(key)
    if arm['read_rule'] == 'first_hit':
        if 'read_leaf_before_buffers' in arm['mutations'] and leaf_entry is not None:
            return {entry_outcome(leaf_entry)}
        if arm['buffer_policy'] is not None:
            for name in ('root_buffer', 'child_buffer'):
                found = newest_in_buffer(arm, state[name], key)
                if found is not None:
                    return found
        return {entry_outcome(leaf_entry)}
    if arm['read_rule'] == 'max_seq':     # 乙定义里「seq 跨两级比较」的字面读法；甲的比 seq 读法
        candidates = [message for message in state['root_buffer'] + state['child_buffer'] if message.key == key]
        if leaf_entry is not None:
            candidates.append(leaf_entry)
        if not candidates:
            return {None}
        top = max(candidate.seq for candidate in candidates)
        return {entry_outcome(candidate) for candidate in candidates if candidate.seq == top}
    raise ValueError(arm['read_rule'])


def read_value(state, arm, key):
    """读的次序：前端（活跃批次 → 冻结批次从新到旧）→ 树。"""
    if 'read_tree_before_front_end' in arm['mutations']:
        found = read_tree(state, arm, key)
        if found != {None}:
            return found
    if key in state['front_end']:
        return {entry_outcome(state['front_end'][key])}
    for batch in reversed(state['frozen_batches']):
        if key in batch:
            return {entry_outcome(batch[key])}
    return read_tree(state, arm, key)


def strip_memory_fields(entry):
    return entry._replace(arrival=OLD, publish=OLD)


def snapshot_durable(state):
    return (tuple(strip_memory_fields(message) for message in state['root_buffer']),
            tuple(strip_memory_fields(message) for message in state['child_buffer']),
            tuple(strip_memory_fields(state['leaf'][key]) for key in sorted(state['leaf'], key=repr)))


def op_publish(state, arm, drain='tree'):
    """发布点：drain='tree' 按臂的 flush 目标把前端排空（甲 / 丙 到叶，乙 到根缓冲）；'none' 不排空。"""
    states = [state] if drain == 'none' else drain_front_end(state, arm)
    for current in states:
        current['durable'] = snapshot_durable(current)
        current['truth_at_publish'] = dict(current['truth'])
        current['current_publish'] += 1
        current['seq_count_in_publish'] = {}
    return states


def reload_durable(state):
    root, child, leaf = state['durable']
    state['root_buffer'] = list(root)
    state['child_buffer'] = list(child)
    state['leaf'] = {entry.key: entry for entry in leaf}
    state['front_end'] = {}
    state['frozen_batches'] = []
    state['seq_count_in_publish'] = {}


def op_crash(state, arm):
    """崩溃：内存全丢，树回到上一次发布；发布号不动（没发出去的那个 txg 重发，D5 已定项 3 第 2 行）。"""
    reload_durable(state)
    state['truth'] = dict(state['truth_at_publish'])
    keys = (set(state['truth']) | set(state['leaf'])
            | {entry.key for entry in state['root_buffer'] + state['child_buffer']})
    for key in keys:
        if read_tree(state, arm, key) != {state['truth'].get(key)}:
            state['recovered_content_wrong'] = True
    return [state]


def universe_of_keys(state):
    keys = set(state['truth']) | set(state['truth_at_publish']) | set(state['front_end']) | set(state['leaf'])
    for batch in state['frozen_batches']:
        keys |= set(batch)
    for name in ('root_buffer', 'child_buffer'):
        keys |= {message.key for message in state[name]}
    return keys


def classify_reads(state, arm):
    """U1：每个 key 读一遍。

    stale           读不到应得的值，路径上没用过任何盘上未定义的平局
    stale_after_tie 读不到应得的值，路径上用过一次盘上未定义的平局（取了对它最不利的那一边）
    undefined       读法本身给出两个答案（同 key 同 seq 而值不同）
    """
    found = {}
    if state['recovered_content_wrong']:
        found['u2_recovered_content'] = None
    for key in sorted(universe_of_keys(state), key=repr):
        outcomes = read_value(state, arm, key)
        expected = state['truth'].get(key)
        if expected not in outcomes:
            label = 'stale_after_tie' if state['tie_broken_adversarially'] else 'stale'
        elif len(outcomes) > 1:
            label = 'undefined'
        else:
            continue
        found.setdefault(label, key)
    return found


def classify_reads_split(state, arm):
    """并行 flush 的形状：前端里还有没排空的批次时读到旧值，与前端排空之后树里就是旧值，分开记。"""
    found = classify_reads(state, arm)
    if has_front_end_content(state):
        return {label + '_front_end_pending': key for label, key in found.items()}
    return found


def canonical(state, arm, relabel_values=True):
    """状态签名：值按首次出现重新编号，到达序与全池 seq 按秩重排，发布号取相对值。只用于去重，不改状态。"""
    current = state['current_publish']
    value_labels = {}

    def value_label(value):
        if value is None or not relabel_values:
            return value
        if value not in value_labels:
            value_labels[value] = len(value_labels)
        return value_labels[value]

    def key_label(key):
        if isinstance(key, tuple) and key[0] in ACCOUNTING_STATISTICS:
            return (key[0], key[1] - current)
        return key

    entries = (list(state['front_end'].values())
               + [entry for batch in state['frozen_batches'] for entry in batch.values()]
               + state['root_buffer'] + state['child_buffer'] + list(state['leaf'].values())
               + [entry for part in state['durable'] for entry in part])
    arrival_rank = {arrival: rank for rank, arrival in
                    enumerate(sorted({entry.arrival for entry in entries if entry.arrival != OLD}))}
    seq_rank = None
    if arm['seq_mode'] != 'per_publish':
        seq_rank = {seq: rank for rank, seq in
                    enumerate(sorted({entry.seq for entry in entries if entry.seq is not None}))}

    def entry_label(entry):
        seq = entry.seq if seq_rank is None or entry.seq is None else seq_rank[entry.seq]
        publish = 'old' if entry.publish == OLD else entry.publish - current
        return (repr(key_label(entry.key)), entry.kind, value_label(entry.value), seq,
                arrival_rank.get(entry.arrival, OLD), publish)

    def mapping_label(mapping, holds_entries):
        ordered = sorted(mapping, key=lambda key: repr(key_label(key)))
        if holds_entries:
            return tuple(entry_label(mapping[key]) for key in ordered)
        return tuple((repr(key_label(key)), value_label(mapping[key])) for key in ordered)

    return (
        mapping_label(state['truth'], False), mapping_label(state['truth_at_publish'], False),
        mapping_label(state['front_end'], True),
        tuple(mapping_label(batch, True) for batch in state['frozen_batches']),
        tuple(entry_label(message) for message in state['root_buffer']),
        tuple(entry_label(message) for message in state['child_buffer']),
        mapping_label(state['leaf'], True),
        tuple(tuple(entry_label(entry) for entry in part) for part in state['durable']),
        tuple(sorted((repr(key_label(key)), count) for key, count in state['seq_count_in_publish'].items())),
        state['recovered_content_wrong'], state['tie_broken_adversarially'],
        tuple(sorted((extent, tag - current) for extent, tag in state['extents'].items())),
        tuple(sorted(state['extents_durable'])),
        None if state['publish_boundary'] is None else state['publish_boundary'] - current,
        state['crash_labels'],
    )


def breadth_first_search(arm, operations, depth_limit, classify, relabel_values=True):
    """按长度逐层穷举操作序列（状态签名去重），记下每类命中第一次出现的最短序列。"""
    start = new_state()
    seen = {canonical(start, arm, relabel_values)}
    frontier = [(start, ())]
    first_hit = {}
    states_explored = 1
    for depth in range(depth_limit + 1):
        for state, path in frontier:
            for label, key in classify(state, arm).items():
                if label not in first_hit:
                    first_hit[label] = (depth, path, key)
        if depth == depth_limit:
            break
        next_frontier = []
        for state, path in frontier:
            for name, operation in operations:
                for successor in operation(copy_state(state), arm):
                    signature = canonical(successor, arm, relabel_values)
                    if signature not in seen:
                        seen.add(signature)
                        next_frontier.append((successor, path + (name,)))
        frontier = next_frontier
        states_explored += len(frontier)
    return first_hit, states_explored


ARM_DEFINITIONS = {
    # 甲（前端 flush 直落叶；节点缓冲区只收不走前端的更新）
    'jia_R1': dict(flush_target='leaf_blind', direct_target='root_buffer', buffer_policy='arrival',
                   read_rule='first_hit', leaf_apply='blind', seq_mode='per_publish'),
    'jia_R2': dict(flush_target='leaf_seq_guard', direct_target='root_buffer', buffer_policy='seq',
                   read_rule='max_seq', leaf_apply='seq_guard', seq_mode='per_publish'),
    'jia_R3': dict(flush_target='leaf_purge', direct_target='root_buffer', buffer_policy='arrival',
                   read_rule='first_hit', leaf_apply='arrival_guard', seq_mode='per_publish'),
    'jia_R4': dict(flush_target='leaf_purge', direct_target='root_buffer', buffer_policy='arrival',
                   read_rule='first_hit', leaf_apply='arrival_guard', seq_mode='per_publish',
                   direct_evicts_front_end=True),
    # 乙（前端 flush 进根缓冲、逐层下推、越靠根越新）。乙的定义里前端树上只有前端这一个写者，
    # 所以默认不走前端的写也进前端；*_two_paths 变体给前端树再开一条直进根缓冲的写路径
    'yi_S': dict(flush_target='root_buffer', direct_target='front_end', buffer_policy='seq',
                 read_rule='first_hit', leaf_apply='blind', seq_mode='per_publish'),
    'yi_X': dict(flush_target='root_buffer', direct_target='front_end', buffer_policy='seq',
                 read_rule='max_seq', leaf_apply='seq_guard', seq_mode='per_publish'),
    'yi_A': dict(flush_target='root_buffer', direct_target='front_end', buffer_policy='arrival',
                 read_rule='first_hit', leaf_apply='blind', seq_mode='per_publish'),
    'yi_D': dict(flush_target='root_buffer', direct_target='front_end', buffer_policy='dedup',
                 read_rule='first_hit', leaf_apply='blind', seq_mode='per_publish'),
    # 丙（一棵树只准一级：前端树 ε = 0，前端树上的写一律进前端）
    'bing': dict(flush_target='leaf_blind', direct_target='front_end', buffer_policy=None,
                 read_rule='first_hit', leaf_apply='blind', seq_mode='per_publish'),
}


def make_arm(name, base=None, **overrides):
    arm = dict(ARM_DEFINITIONS[base or name])
    arm['mutations'] = frozenset()
    arm.update(overrides)
    arm['name'] = name
    return arm


def generic_operations():
    return [
        ('fe_put', lambda state, arm: front_end_update(state, GENERIC_KEY, 'P')),
        ('fe_del', lambda state, arm: front_end_update(state, GENERIC_KEY, 'D')),
        ('direct_put', lambda state, arm: direct_update(state, arm, GENERIC_KEY, 'P')),
        ('direct_del', lambda state, arm: direct_update(state, arm, GENERIC_KEY, 'D')),
        ('flush', op_flush), ('push_R', op_push_root), ('push_C', op_push_child),
        ('publish', op_publish), ('crash', op_crash),
    ]


RETAINED_GENERATIONS = 1   # K 取 1 让序列最短；K 只决定删在第几次发布，不改形状（D5 已定项 2 的点删）


def accounting_operations():
    def current_key(state):
        return ('s', state['current_publish'])

    def discard(route):
        def operation(state, arm):
            key = ('s', state['current_publish'] - RETAINED_GENERATIONS)
            if key[1] < 0 or key not in state['truth']:
                return []
            if route == 'front_end':
                return front_end_update(state, key, 'D')
            return direct_update(state, arm, key, 'D')
        return operation

    return [
        ('acc_put', lambda state, arm: front_end_update(state, current_key(state), 'P')),
        ('acc_discard_fe', discard('front_end')), ('acc_discard_direct', discard('direct')),
        ('flush', op_flush), ('push_R', op_push_root), ('push_C', op_push_child),
        ('publish', op_publish), ('crash', op_crash),
    ]


def parallel_operations(allow_out_of_order):
    operations = [
        ('fe_put', lambda state, arm: front_end_update(state, GENERIC_KEY, 'P')),
        ('freeze', op_freeze),
        ('flush_oldest_batch', lambda state, arm: op_flush_batch(state, arm, 0)),
        ('push_R', op_push_root), ('push_C', op_push_child),
        ('publish', op_publish),
    ]
    if allow_out_of_order:
        operations.append(('flush_newest_batch',
                           lambda state, arm: op_flush_batch(state, arm, -1)
                           if len(state['frozen_batches']) >= 2 else []))
    return operations


def check_accounting_after_crash(state, arm):
    """I-3.1：恢复出的「已分配」统计（取最新那一代）== 恢复出的 extent 数；反向索引条目集 == extent 集。"""
    labels = set()
    allocated_by_generation = {}
    reverse_index = set()
    keys = {entry.key for entry in state['root_buffer'] + state['child_buffer']} | set(state['leaf'])
    for key in keys:
        outcomes = read_tree(state, arm, key)
        if len(outcomes) != 1:
            labels.add('undefined_after_crash')
            continue
        (value,) = outcomes
        if value is None:
            continue
        if key[0] == 'alloc':
            allocated_by_generation[key[1]] = value
        elif key[0] == 'ri':
            reverse_index.add(key[1])
    durable_extents = set(state['extents_durable'])
    allocated = allocated_by_generation[max(allocated_by_generation)] if allocated_by_generation else 0
    if allocated != len(durable_extents):
        labels.add('I-3.1')
    if reverse_index != durable_extents:
        labels.add('reverse_index')
    return labels


def classify_crash_labels(state, arm):
    return {label: None for label in sorted(state['crash_labels'])}


def u2_operations(tagging, drain):
    """U2：extent 直落（不走前端），反向索引与「已分配」记账走前端。发布可以一步做完，也可以拆成
    publish_begin（划边界）… publish_end（排空并落盘）两步，中间到达的写属于下一个窗口。
    tagging=True：前端按窗口打标签，排空只取不晚于边界的；False：排空取前端里的全部。"""
    def op_write(state, arm):
        state['extent_counter'] += 1
        extent = state['extent_counter']
        boundary = state['publish_boundary']
        tag = state['current_publish'] if boundary is None else boundary + 1
        state['extents'][extent] = tag
        allocated = sum(1 for extent_tag in state['extents'].values() if extent_tag <= tag)
        front_end_update(state, ('ri', extent), 'P', value=1, tag=tag)
        front_end_update(state, ('alloc', tag), 'P', value=allocated, tag=tag)
        return [state]

    def op_u2_flush(state, arm):
        if not has_front_end_content(state):
            return []
        return drain_front_end(state, arm, state['publish_boundary'] if tagging else None)

    def op_publish_begin(state, arm):
        if state['publish_boundary'] is not None:
            return []
        state['publish_boundary'] = state['current_publish']
        return [state]

    def op_publish_end(state, arm):
        boundary = state['publish_boundary']
        if boundary is None:
            return []
        states = [state] if drain == 'none' else drain_front_end(state, arm, boundary if tagging else None)
        for current in states:
            current['durable'] = snapshot_durable(current)
            current['extents_durable'] = {extent: tag for extent, tag in current['extents'].items() if tag <= boundary}
            current['current_publish'] = boundary + 1
            current['publish_boundary'] = None
            current['seq_count_in_publish'] = {}
        return states

    def op_publish_atomic(state, arm):
        return [after for begun in op_publish_begin(state, arm) for after in op_publish_end(begun, arm)]

    def op_u2_crash(state, arm):
        if state['publish_boundary'] is not None:
            state['current_publish'] = state['publish_boundary']
            state['publish_boundary'] = None
        reload_durable(state)
        state['extents'] = dict(state['extents_durable'])
        state['crash_labels'] = state['crash_labels'] | frozenset(check_accounting_after_crash(state, arm))
        return [state]

    return [('write', op_write), ('flush', op_u2_flush), ('push_R', op_push_root), ('push_C', op_push_child),
            ('publish', op_publish_atomic), ('publish_begin', op_publish_begin),
            ('publish_end', op_publish_end), ('crash', op_u2_crash)]


PAIR_SCRIPTS = {
    # C161 那一行的检查形态：同一个 key 两条，较旧的先写、较新的后写；None = 这一族臂上摆不出来
    'front_end+root_buffer': {'yi': ['fe_put', 'flush', 'fe_put'], 'jia': ['direct_put', 'fe_put'], 'bing': None},
    'root_buffer+child_buffer': {'yi': ['fe_put', 'flush', 'push_R', 'fe_put', 'flush'],
                                 'jia': ['direct_put', 'push_R', 'direct_put'], 'bing': None},
    'root_buffer+leaf': {'yi': ['fe_put', 'flush', 'push_R', 'push_C', 'fe_put', 'flush'],
                         'jia': ['fe_put', 'flush', 'direct_put'], 'bing': None},
    'front_end+leaf': {'yi': ['fe_put', 'flush', 'push_R', 'push_C', 'fe_put'],
                       'jia': ['fe_put', 'flush', 'fe_put'], 'bing': ['fe_put', 'flush', 'fe_put']},
    'same_buffer_same_publish': {'yi': ['fe_put', 'flush', 'fe_put', 'flush'],
                                 'jia': ['direct_put', 'direct_put'], 'bing': None},
    'same_buffer_across_publish': {'yi': ['fe_put', 'publish', 'fe_put', 'publish'],
                                   'jia': ['direct_put', 'publish', 'direct_put', 'publish'], 'bing': None},
}


def arm_family(arm):
    if arm['flush_target'] == 'root_buffer':
        return 'yi'
    if arm['buffer_policy'] is None:
        return 'bing'
    return 'jia'


def run_script(arm, script):
    operations = dict(generic_operations())
    state = new_state()
    for name in script:
        successors = operations[name](copy_state(state), arm)
        if not successors:
            return None
        state = successors[0]
    return state


def locate_key_entries(state, key):
    places = []
    if key in state['front_end']:
        places.append(('front_end', key))
    for name in ('root_buffer', 'child_buffer'):
        places.extend((name, index) for index, message in enumerate(state[name]) if message.key == key)
    if key in state['leaf']:
        places.append(('leaf', key))
    return places


def swap_check(arm, pair):
    """判别力自证：检查「读出来的是新的那条」先绿；把两条的新旧（值）对调必须转红；只对调 seq 看这条臂读不读 seq。"""
    script = PAIR_SCRIPTS[pair][arm_family(arm)]
    state = run_script(arm, script) if script is not None else None
    places = locate_key_entries(state, GENERIC_KEY) if state is not None else []
    if len(places) != 2:
        return 'unreachable', '-', '-'
    expected = {state['truth'].get(GENERIC_KEY)}
    before = 'green' if read_value(state, arm, GENERIC_KEY) == expected else 'red_before_fault'
    (first_name, first_index), (second_name, second_index) = places
    first_entry, second_entry = state[first_name][first_index], state[second_name][second_index]
    value_swapped = copy_state(state)
    value_swapped[first_name][first_index] = first_entry._replace(kind=second_entry.kind, value=second_entry.value)
    value_swapped[second_name][second_index] = second_entry._replace(kind=first_entry.kind, value=first_entry.value)
    after_value_swap = 'red' if read_value(value_swapped, arm, GENERIC_KEY) != expected else 'green'
    if first_name == 'front_end':
        return before, after_value_swap, 'n/a'
    seq_swapped = copy_state(state)
    seq_swapped[first_name][first_index] = first_entry._replace(seq=second_entry.seq)
    seq_swapped[second_name][second_index] = second_entry._replace(seq=first_entry.seq)
    after_seq_swap = 'red' if read_value(seq_swapped, arm, GENERIC_KEY) != expected else 'green'
    return before, after_value_swap, after_seq_swap


NODE_BYTES = 16384   # D8 已定项 2


def fanout_for(epsilon, pivot_bytes):
    return int(NODE_BYTES * (1 - epsilon) // pivot_bytes)      # E56 的式子 F(ε) = floor(S × (1−ε) / P)


def buffer_messages_for(epsilon, message_bytes):
    return int(NODE_BYTES * epsilon // message_bytes)          # E56 的式子 B(ε) = floor(S × ε / M)


def tree_height(leaf_count, fanout):
    height, nodes = 1, leaf_count
    while nodes > 1:
        nodes = -(-nodes // fanout)
        height += 1
    return height


def nodes_written_drain_to_leaf(leaf_count, fanout, entries_per_publish, publishes, seed):
    """丙 / 甲 的前端树（ε = 0）：每次发布把前端排空到叶，写出的节点 = 碰到的叶 + 全部祖先（一次发布里去重）。"""
    generator = random.Random(seed)
    height = tree_height(leaf_count, fanout)
    total = 0
    for _ in range(publishes):
        written = set()
        for _ in range(entries_per_publish):
            node = generator.randrange(leaf_count)
            for level in range(height):
                written.add((level, node))
                node //= fanout
        total += len(written)
    return total / publishes, height


def nodes_written_drain_to_root_buffer(leaf_count, fanout, buffer_capacity, entries_per_publish, publishes, seed):
    """乙（ε > 0）：每次发布把前端排空进根缓冲；任一内部节点的缓冲超过容量就把消息最多的那个孩子整份下推（级联）。
    一次发布里同一个节点只写出一次（D19 已定项 12 的固定点）。先按每次 256 条灌到全树缓冲容量的 2 倍再开始计数。"""
    generator = random.Random(seed)
    height = tree_height(leaf_count, fanout)
    root = (height - 1, 0)
    buffers = {}

    def child_index(level, leaf):
        return leaf // (fanout ** (level - 1))

    def overflow(node, written):
        level = node[0]
        buffer = buffers.setdefault(node, {})
        while sum(len(targets) for targets in buffer.values()) > buffer_capacity:
            child = max(buffer, key=lambda index: (len(buffer[index]), -index))
            targets = buffer.pop(child)
            child_node = (level - 1, child)
            written.add(child_node)
            if level - 1 == 0:
                continue
            child_buffer = buffers.setdefault(child_node, {})
            for leaf in targets:
                child_buffer.setdefault(child_index(level - 1, leaf), []).append(leaf)
            overflow(child_node, written)

    def one_publish(entry_count):
        written = {root}
        root_buffer = buffers.setdefault(root, {})
        for _ in range(entry_count):
            leaf = generator.randrange(leaf_count)
            root_buffer.setdefault(child_index(height - 1, leaf), []).append(leaf)
        overflow(root, written)
        return len(written)

    internal_nodes = sum(-(-leaf_count // (fanout ** level)) for level in range(1, height))
    for _ in range(2 * internal_nodes * buffer_capacity // 256 + 1):
        one_publish(256)
    total = sum(one_publish(entries_per_publish) for _ in range(publishes))
    return total / publishes, height


MUTATIONS = [
    # (编号, 被改坏的臂, 改坏了什么)
    ('M1', 'bing', dict(mutations=frozenset({'read_tree_before_front_end'})), '丙：先读树再读前端'),
    ('M2', 'bing', dict(direct_target='leaf_blind'), '丙：前端树上有绕开前端直落叶的写者'),
    ('M3', 'yi_D', dict(mutations=frozenset({'dedup_keeps_old'})), '乙-D：缓冲去重留旧丢新'),
    ('M4', 'yi_D', dict(mutations=frozenset({'push_keeps_child'})), '乙-D：下推时孩子的同 key 旧消息不被替换'),
    ('M5', 'yi_D', dict(mutations=frozenset({'read_leaf_before_buffers'})), '乙-D：先读叶再读缓冲'),
    ('M6', 'yi_A', dict(mutations=frozenset({'push_inserts_before'})), '乙-A：下推来的消息排到孩子已有消息前面'),
    ('M7', 'jia_R4', dict(mutations=frozenset({'purge_skipped'})), '甲-R4：flush 不清路径上的旧消息'),
    ('M8', 'jia_R4', dict(mutations=frozenset({'purge_leaf_blind'})), '甲-R4：flush 落叶不比同窗口到达序'),
    ('M9', 'jia_R4', dict(direct_evicts_front_end=False), '甲-R4：不走前端的写不逐出前端里的同 key'),
    ('M10', 'yi_D', dict(direct_target='root_buffer'), '乙-D：前端树上再开一条直进根缓冲的写路径'),
]
# 甲-R4 的「逐出前端」让顺序负载上每条前端条目都比树里同 key 的新，M8 在那里在所有输入上与原式同值；
# 敏感的取样点在乱序 flush 那个形状上（mutation-sampling.md 的第三类：换一个取样点）。
MUTATION_WORKLOAD = {'M8': 'parallel_out_of_order'}


NAMED_SHAPES = [
    # (编号, 负载, 操作序列, 读哪个 key)：把反向接受条款点名的形状与腿自己要攻的形状逐条跑一遍
    ('S1_buffer_old_shadows_leaf_new', 'generic', ['direct_put', 'fe_put', 'flush'], GENERIC_KEY),
    ('S2_front_end_old_shadows_direct_new', 'generic', ['fe_put', 'direct_put'], GENERIC_KEY),
    ('S3_push_overwrites_newer_leaf', 'generic', ['direct_put', 'fe_put', 'flush', 'push_R', 'push_C'], GENERIC_KEY),
    ('S4_same_buffer_tie_across_publish', 'generic', ['fe_put', 'publish', 'fe_put', 'publish'], GENERIC_KEY),
    ('S5_same_buffer_seq_inversion_across_publish', 'generic',
     ['fe_put', 'flush', 'fe_put', 'flush', 'publish', 'fe_put', 'flush'], GENERIC_KEY),
    ('S6_cross_level_seq_inversion', 'generic',
     ['fe_put', 'flush', 'fe_put', 'flush', 'publish', 'push_R', 'push_C', 'fe_put', 'flush'], GENERIC_KEY),
    ('S7_discard_tombstone_tie', 'accounting', ['acc_put', 'publish', 'acc_discard_fe', 'publish'], ('s', 0)),
    ('S8_discard_tombstone_inversion', 'accounting',
     ['acc_put', 'flush', 'acc_put', 'publish', 'acc_discard_fe', 'flush'], ('s', 0)),
    ('S9_tie_survives_crash', 'generic', ['fe_put', 'publish', 'fe_put', 'publish', 'crash'], GENERIC_KEY),
]


def run_shape(arm, workload, script, key):
    """跑一条写死的序列；平局分叉的每一支都判。返回 ok / stale / undefined 的并集，或 not_applicable。"""
    operations = dict(generic_operations() if workload == 'generic' else accounting_operations())
    states = [new_state()]
    for name in script:
        states = [successor for state in states for successor in operations[name](copy_state(state), arm)]
        if not states:
            return 'not_applicable'
    verdicts = set()
    for state in states:
        outcomes = read_value(state, arm, key)
        expected = state['truth'].get(key)
        if expected not in outcomes:
            verdicts.add('stale')
        elif len(outcomes) > 1:
            verdicts.add('undefined')
        else:
            verdicts.add('ok')
    return '+'.join(sorted(verdicts))


def format_hit(first_hit, label):
    if label not in first_hit:
        return '-'
    depth, path, key = first_hit[label]
    where = '|after_crash' if key is None else '|READ(' + repr(key).replace(' ', '') + ')'
    return f"{depth}:{','.join(path) if path else '(empty)'}{where}"


def u1_arms():
    return [
        make_arm('jia_R1'), make_arm('jia_R2'), make_arm('jia_R2_global', 'jia_R2', seq_mode='global'),
        make_arm('jia_R3'), make_arm('jia_R4'),
        make_arm('yi_S'), make_arm('yi_X'), make_arm('yi_S_global', 'yi_S', seq_mode='global'),
        make_arm('yi_X_global', 'yi_X', seq_mode='global'), make_arm('yi_A'), make_arm('yi_D'),
        make_arm('yi_S_two_paths', 'yi_S', direct_target='root_buffer'),
        make_arm('yi_D_two_paths', 'yi_D', direct_target='root_buffer'),
        make_arm('bing'),
        make_arm('jia_R4_adversarial_seq', 'jia_R4', seq_mode='adversarial'),
        make_arm('yi_A_adversarial_seq', 'yi_A', seq_mode='adversarial'),
        make_arm('yi_D_adversarial_seq', 'yi_D', seq_mode='adversarial'),
        make_arm('bing_adversarial_seq', 'bing', seq_mode='adversarial'),
    ]


def main():
    depth_u1 = int(sys.argv[1]) if len(sys.argv) > 1 else 8
    depth_u2 = int(sys.argv[2]) if len(sys.argv) > 2 else 6
    lines = []
    labels = ('stale', 'undefined', 'stale_after_tie', 'u2_recovered_content')
    for workload_name, operations in (('generic', generic_operations()), ('accounting', accounting_operations())):
        for arm in u1_arms():
            first_hit, states = breadth_first_search(arm, operations, depth_u1, classify_reads)
            hits = ' '.join(f"{label}={format_hit(first_hit, label)}" for label in labels)
            lines.append(f"C161 name=u1 workload={workload_name} arm={arm['name']} seq={arm['seq_mode']} "
                         f"depth_limit={depth_u1} states={states} {hits}")
    parallel_arms = [
        make_arm('bing'), make_arm('bing_seq_guard', 'bing', flush_target='leaf_seq_guard'),
        make_arm('bing_arrival_guard', 'bing', flush_target='leaf_arrival_guard'),
        make_arm('yi_S'), make_arm('yi_D'), make_arm('yi_D_arrival_guard', 'yi_D', buffer_policy='dedup_arrival_guard'),
        make_arm('jia_R4'),
    ]
    parallel_labels = ('stale', 'stale_after_tie', 'undefined', 'stale_front_end_pending')
    for allow_out_of_order in (False, True):
        for arm in parallel_arms:
            first_hit, states = breadth_first_search(arm, parallel_operations(allow_out_of_order), depth_u1,
                                                     classify_reads_split)
            hits = ' '.join(f"{label}={format_hit(first_hit, label)}" for label in parallel_labels)
            lines.append(f"C161 name=u1_parallel_flush out_of_order={allow_out_of_order} arm={arm['name']} "
                         f"depth_limit={depth_u1} states={states} {hits}")
    u2_arm_names = ('jia_R1', 'jia_R2', 'jia_R3', 'jia_R4', 'yi_S', 'yi_X', 'yi_A', 'yi_D', 'bing')
    for drain, tagging in (('tree', True), ('none', True), ('tree', False)):
        for arm_name in u2_arm_names:
            arm = make_arm(arm_name)
            first_hit, states = breadth_first_search(arm, u2_operations(tagging, drain), depth_u2,
                                                     classify_crash_labels, relabel_values=False)
            hits = ' '.join(f"{label}={format_hit(first_hit, label)}"
                            for label in ('I-3.1', 'reverse_index', 'undefined_after_crash'))
            lines.append(f"C161 name=u2 drain={drain} front_end_tagged={tagging} arm={arm_name} "
                         f"depth_limit={depth_u2} states={states} {hits}")
    for arm in [candidate for candidate in u1_arms() if candidate['seq_mode'] != 'adversarial']:
        for shape_name, workload, script, key in NAMED_SHAPES:
            verdict = run_shape(arm, workload, script, key)
            lines.append(f"C161 name=u1_shape arm={arm['name']} shape={shape_name} verdict={verdict}")
    for arm_name in ('jia_R1', 'jia_R2', 'jia_R3', 'jia_R4', 'yi_S', 'yi_X', 'yi_A', 'yi_D', 'bing'):
        arm = make_arm(arm_name)
        for pair in PAIR_SCRIPTS:
            before, after_value_swap, after_seq_swap = swap_check(arm, pair)
            lines.append(f"C161 name=u4_swap arm={arm_name} pair={pair} before={before} "
                         f"after_value_swap={after_value_swap} after_seq_swap={after_seq_swap}")
    for arm_name in ('jia_R4', 'yi_A', 'yi_D', 'bing'):
        first_hit, states = breadth_first_search(make_arm(arm_name), generic_operations(), depth_u1, classify_reads)
        caught = any(label in first_hit for label in ('stale', 'undefined', 'stale_after_tie'))
        lines.append(f"C161 name=u4_negative_control arm={arm_name} depth_limit={depth_u1} states={states} "
                     f"any_hit={caught}")
    for number, base, overrides, description in MUTATIONS:
        arm = make_arm(f"{base}+{number}", base, **overrides)
        workload = MUTATION_WORKLOAD.get(number, 'generic')
        if workload == 'generic':
            operations, classify = generic_operations(), classify_reads
        else:
            operations, classify = parallel_operations(True), classify_reads_split
        first_hit, states = breadth_first_search(arm, operations, depth_u1, classify)
        found = [label for label in ('stale', 'stale_after_tie', 'undefined') if label in first_hit]
        shortest = format_hit(first_hit, found[0]) if found else '-'
        lines.append(f"C161 name=u4_mutation id={number} arm={base} workload={workload} depth_limit={depth_u1} "
                     f"caught={bool(found)} label={found[0] if found else '-'} "
                     f"shortest={shortest} what={description.replace(' ', '_')}")
    width_points = (('E56_widths', 48, 16), ('today_widths', 117, 34))
    for width_label, pivot_bytes, message_bytes in width_points:
        fanout_plain = fanout_for(0.0, pivot_bytes)
        fanout_buffered = fanout_for(0.65, pivot_bytes)
        capacity = buffer_messages_for(0.65, message_bytes)
        for leaf_count in (1024, 32768):
            for entries_per_publish in (1, 2, 16, 256, 4096):
                publishes = 2000 if entries_per_publish <= 16 else (300 if entries_per_publish <= 256 else 40)
                leaf_cost, leaf_height = nodes_written_drain_to_leaf(leaf_count, fanout_plain, entries_per_publish,
                                                                      publishes, seed=161)
                buffer_cost, buffer_height = nodes_written_drain_to_root_buffer(
                    leaf_count, fanout_buffered, capacity, entries_per_publish, publishes, seed=161)
                lines.append(f"C161 name=u3_nodes_per_publish widths={width_label} pivot={pivot_bytes} "
                             f"message={message_bytes} leaves={leaf_count} entries_per_publish={entries_per_publish} "
                             f"publishes={publishes} drain_to_leaf(F={fanout_plain},h={leaf_height})={leaf_cost:.3f} "
                             f"drain_to_root_buffer(F={fanout_buffered},B={capacity},h={buffer_height})={buffer_cost:.3f} "
                             f"ratio={leaf_cost / buffer_cost:.3f}")
    lines.append(f"emitted={len(lines)}")
    print('\n'.join(lines))


if __name__ == '__main__':
    main()
