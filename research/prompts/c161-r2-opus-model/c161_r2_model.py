#!/usr/bin/env python3
"""C161（缓冲两级并存而衔接没定） 第二轮攻方腿（Opus）的模型，只用 Python 标准库。

第一轮攻方腿的模型（research/prompts/c161-r1-opus-model/c161_model.py）只有一条路径、没有分裂合并、K = 1、
没有部分发布。这一轮换攻击面，分四块：
  A  两棵子树的树（高度 2 ↔ 3）：grow（根分裂、树长高）、shrink（根塌缩、树变矮）、rebalance（边界 key 在两棵子树
     之间搬家）时缓冲内容怎么重分配；同一批次的 key 落在不同子树、只下推一棵子树（部分下推）；
     B  K > 1 的记账点删；seq 全池单调时计数器在崩溃后怎么接
  C  fsync 的部分发布（只写被 fsync 的那个文件的脏叶 + 祖先）与发布集的边界；排空跨两次发布（排空做到一半崩溃）
  E  陈旧检出（D11 已定项 3、C52）要的盘上时间线索，与各臂的新旧判定比
  F  U3：全池单调 seq 下消息在缓冲里的停留期（按全池写入次数计）；部分发布下排空到叶的写出量
复跑：python3 -B c161_r2_model.py > c161-r2-model-output.txt（产物末行 emitted=N，行数对不上即作废）。
"""
import bisect
import itertools
import random
import sys
from collections import namedtuple

# 一条条目。kind 'P' 写入 / 'D' 墓碑；value 每次写入一个新值；seq 盘上的序号；arrival 内存里的到达序（时间序的真值）；
# publish 进前端或进树时的发布号（内存里的窗口标签）；txg 写进树的那次发布的号（盘上，只给陈旧检出的检测器用）；
# origin 'fe' 从前端排空来、'direct' 不走前端的写。arrival / publish 从盘上重读之后丢失，记 OLD。
Entry = namedtuple('Entry', 'key kind value seq arrival publish txg origin')
OLD = -1


def entry_outcome(entry):
    """一条条目读出来的值：墓碑与不存在都读成 None。"""
    if entry is None:
        return None
    return entry.value if entry.kind == 'P' else None


def route(view, key):
    """高度 3 时 key 进哪个内部节点。通用 key 按 pivot 分：pivot='b' ⇒ b 在子树 1，'c' ⇒ b 在子树 0（a 恒在子树 0）；
    记账 key 按代的奇偶分（相邻两代落在不同子树）。"""
    if isinstance(key, tuple):
        return key[1] % 2
    return 0 if key < view['pivot'] else 1


def empty_durable():
    return (2, 'b', (), ((), ()), ())


def new_state():
    return {
        'front_end': {}, 'height': 2, 'pivot': 'b', 'root': [], 'inner': [[], []], 'leaf': {},
        'durable': empty_durable(), 'truth': {}, 'truth_at_publish': {},
        'current_publish': 0, 'seq_count_in_publish': {}, 'global_seq': 0, 'durable_global_seq': 0,
        'arrival_counter': 0, 'fresh_value_counter': 0,
        'tie': False, 'recovered_wrong': False, 'restructure_changed_read': False, 'second_writer_seen': False,
    }


def copy_state(state):
    copied = dict(state)
    for field in ('front_end', 'leaf', 'truth', 'truth_at_publish', 'seq_count_in_publish'):
        copied[field] = dict(state[field])
    copied['root'] = list(state['root'])
    copied['inner'] = [list(state['inner'][0]), list(state['inner'][1])]
    return copied


def assign_seq(state, arm, key):
    """盘上 seq 的两种口径：本次发布里被写进树的第几次（D8 已定项 10），或全池单调。"""
    if arm['seq_mode'] == 'per_publish':
        count = state['seq_count_in_publish'].get(key, 0) + 1
        state['seq_count_in_publish'][key] = count
        return count
    state['global_seq'] += 1
    return state['global_seq']


def record_truth(state, key, kind):
    """按时间序记真值；返回这次写入的值与到达序。"""
    state['arrival_counter'] += 1
    if kind == 'P':
        state['fresh_value_counter'] += 1
        value = state['fresh_value_counter']
        state['truth'][key] = value
    else:
        value = None
        state['truth'].pop(key, None)
    return value, state['arrival_counter']


def front_end_update(state, arm, key, kind):
    """写进前端：前端在内存里按到达序去重（后到的替换先到的）。全池单调、到达即取号的臂在这里取 seq。"""
    value, arrival = record_truth(state, key, kind)
    seq = assign_seq(state, arm, key) if arm['seq_assign'] == 'arrival' else None
    state['front_end'][key] = Entry(key, kind, value, seq, arrival, state['current_publish'], None, 'fe')
    return [state]


def buffer_insert(arm, buffer, entry):
    """把一条消息放进某一层缓冲（buffer 是 list，原地改）。"""
    policy = arm['buffer_policy']
    if policy == 'dedup':             # 乙-D：一个缓冲区里同 key 至多一条，新来的替换旧的
        buffer[:] = [message for message in buffer if message.key != entry.key] + [entry]
        return
    if policy in ('append', 'seq'):   # 甲-R4：位置即新旧（后放进来的新）；乙 + seq：同一层由 seq 定
        buffer.append(entry)
        return
    raise ValueError(policy)


def direct_update(state, arm, key, kind):
    """不走前端的写。乙、丙：前端树上没有这类写者，一律进前端；甲-R4：进根缓冲，并逐出前端里的同 key。"""
    target = arm['direct_target']
    if target == 'front_end':
        return front_end_update(state, arm, key, kind)
    value, arrival = record_truth(state, key, kind)
    entry = Entry(key, kind, value, assign_seq(state, arm, key), arrival, state['current_publish'],
                  state['current_publish'], 'direct')
    if arm['direct_evicts_front_end']:
        state['front_end'].pop(key, None)
    if target == 'leaf_blind':        # 变异 N4：丙的前端树上多一个绕开前端、直落叶的写者（第一轮 M2）
        state['leaf'][key] = entry
        return [state]
    if target == 'root':
        buffer_insert(arm, state['root'], entry)
        return [state]
    raise ValueError(target)


def newest_in_buffer(arm, buffer, key):
    same_key = [message for message in buffer if message.key == key]
    if not same_key:
        return None
    if arm['buffer_policy'] == 'seq':
        top = max(message.seq for message in same_key)
        return {entry_outcome(message) for message in same_key if message.seq == top}
    return {entry_outcome(same_key[-1])}


def read_tree(view, arm, key):
    """只读树：根缓冲 → （高度 3 时）key 所在内部节点的缓冲 → 叶，取第一条命中。view 可以是状态，也可以是盘上镜像。"""
    if arm['buffer_policy'] is not None:
        found = newest_in_buffer(arm, view['root'], key)
        if found is not None:
            return found
        if view['height'] == 3:
            found = newest_in_buffer(arm, view['inner'][route(view, key)], key)
            if found is not None:
                return found
    return {entry_outcome(view['leaf'].get(key))}


def read_value(state, arm, key):
    """读的次序：前端 → 树。"""
    if key in state['front_end']:
        return {entry_outcome(state['front_end'][key])}
    return read_tree(state, arm, key)


def arrival_older(state, older, newer):
    """只用内存信息判 older 是不是比 newer 旧：本窗口没发布的一律比已发布的新；两条都已发布时判不出（None）。"""
    current = state['current_publish']

    def is_unpublished(entry):
        return entry.arrival != OLD and entry.publish >= current

    older_is_unpublished, newer_is_unpublished = is_unpublished(older), is_unpublished(newer)
    if older_is_unpublished and newer_is_unpublished:
        return older.arrival < newer.arrival
    if newer_is_unpublished:
        return True
    if older_is_unpublished:
        return False
    return None


def leaf_apply_one(state, entry, rule):
    existing = state['leaf'].get(entry.key)
    if existing is None or rule == 'blind':
        state['leaf'][entry.key] = entry
        return [state]
    if rule == 'arrival_guard':       # 丙 / 甲-R4：落叶比同一窗口的到达序；叶里那条更新就不覆盖
        if arrival_older(state, existing, entry) is False:
            return [state]
        state['leaf'][entry.key] = entry
        return [state]
    raise ValueError(rule)


def seq_orderings(messages):
    """seq 口径的缓冲下推到叶时按 seq 施加；同 key 同 seq 而值不同的，盘上次序没有定义——每种排列都走一遍。"""
    groups = {}
    for message in messages:
        groups.setdefault((repr(message.key), message.seq), []).append(message)
    per_group = []
    tie = False
    for group_key in sorted(groups):
        group = groups[group_key]
        if len({entry_outcome(message) for message in group}) > 1:
            tie = True
            per_group.append(list(itertools.permutations(group)))
        else:
            per_group.append([tuple(group)])
    orderings = [[message for group in combination for message in group]
                 for combination in itertools.product(*per_group)]
    return orderings, tie


def apply_messages_to_leaf(state, arm, messages):
    """缓冲里的消息落叶：位置口径按缓冲里的次序盲写（缓冲里的比叶新）；seq 口径按 seq 施加。"""
    if arm['buffer_policy'] == 'seq':
        orderings, tie = seq_orderings(messages)
    else:
        orderings, tie = [list(messages)], False
    branches = [state] + [copy_state(state) for _ in orderings[1:]]
    for branch, ordering in zip(branches, orderings):
        if tie:
            branch['tie'] = True
        for message in ordering:
            branch['leaf'][message.key] = message
    return branches


def purge_path(state, arm, entry):
    """甲-R4：flush 时清掉路径上比它旧的同 key 消息。路径按当前 pivot 算（变异 N3 走另一棵子树）。"""
    def keep(message):
        return not (message.key == entry.key and arrival_older(state, message, entry))

    state['root'] = [message for message in state['root'] if keep(message)]
    if state['height'] == 3:
        index = route(state, entry.key)
        if arm['purge_route'] == 'stale':
            index = 1 - index
        state['inner'][index] = [message for message in state['inner'][index] if keep(message)]


def flush_one(state, arm, front_entry):
    """前端的一条进树：没在到达时取号的，此刻取（D8 已定项 10：被写进树的第几次）；txg 记这次发布号。"""
    seq = front_entry.seq if front_entry.seq is not None else assign_seq(state, arm, front_entry.key)
    entry = front_entry._replace(seq=seq, txg=state['current_publish'])
    target = arm['flush_target']
    if target == 'root':
        buffer_insert(arm, state['root'], entry)
        return [state]
    if target == 'leaf_arrival_guard':
        return leaf_apply_one(state, entry, 'arrival_guard')
    if target == 'leaf_purge':
        purge_path(state, arm, entry)
        return leaf_apply_one(state, entry, 'arrival_guard')
    raise ValueError(target)


def flush_front_end(state, arm):
    """前端全部进树；批次内按 key 排序（排序丢掉时间序），同一批里 a、b 可以落在不同子树。"""
    entries = [state['front_end'][key] for key in sorted(state['front_end'], key=repr)]
    state['front_end'] = {}
    states = [state]
    for entry in entries:
        states = [successor for current in states for successor in flush_one(current, arm, entry)]
    return states


def op_flush(state, arm):
    return flush_front_end(state, arm) if state['front_end'] else []


def op_push_root(state, arm, which=None):
    """高度 2：根缓冲下推到叶；高度 3：根缓冲按路由下推到内部节点；which 给了只推那一棵子树的消息（部分下推）。"""
    if arm['buffer_policy'] is None or not state['root']:
        return []
    if state['height'] == 2:
        if which is not None:
            return []
        messages = state['root']
        state['root'] = []
        return apply_messages_to_leaf(state, arm, messages)

    def selected(message):
        return which is None or route(state, message.key) == which

    moving = [message for message in state['root'] if selected(message)]
    if not moving:
        return []
    state['root'] = [message for message in state['root'] if not selected(message)]
    for message in moving:
        buffer_insert(arm, state['inner'][route(state, message.key)], message)
    return [state]


def op_push_inner(state, arm, index):
    if state['height'] != 3 or arm['buffer_policy'] is None or not state['inner'][index]:
        return []
    messages = state['inner'][index]
    state['inner'][index] = []
    return apply_messages_to_leaf(state, arm, messages)


def universe_of_keys(state):
    keys = set(state['truth']) | set(state['truth_at_publish']) | set(state['front_end']) | set(state['leaf'])
    for message in state['root'] + state['inner'][0] + state['inner'][1]:
        keys.add(message.key)
    return keys


def reads_snapshot(state, arm):
    return tuple((repr(key), frozenset(read_value(state, arm, key)))
                 for key in sorted(universe_of_keys(state), key=repr))


def finish_restructure(state, arm, before):
    """检查 G1：重组前后每个 key 读出来的值不变（重组不是写，读的结果不许变）。"""
    if reads_snapshot(state, arm) != before:
        state['restructure_changed_read'] = True
    return [state]


def op_grow(state, arm):
    """根分裂、树长高：原来那个节点（叶的上一层）按 pivot 裂成两个内部节点，各带走自己那一半缓冲；新根的缓冲为空。"""
    if state['height'] != 2:
        return []
    before = reads_snapshot(state, arm)
    state['inner'] = [[message for message in state['root'] if route(state, message.key) == 0],
                      [message for message in state['root'] if route(state, message.key) == 1]]
    state['root'] = []
    state['height'] = 3
    return finish_restructure(state, arm, before)


def op_shrink(state, arm):
    """根塌缩、树变矮：两个内部节点与根并成一个节点。
    push_down：先把根缓冲按下推规则推进两个孩子，再把两个孩子并成新根（越靠根越新保住）；
    fold_up：留下的是原来的根节点，孩子缓冲里的消息当「新来的」按插入规则放进根缓冲——臂的定义没说塌缩怎么做时的最弱读法。"""
    if state['height'] != 3:
        return []
    before = reads_snapshot(state, arm)
    if arm['collapse'] == 'push_down':
        for message in state['root']:
            buffer_insert(arm, state['inner'][route(state, message.key)], message)
        state['root'] = state['inner'][0] + state['inner'][1]
    else:
        for message in state['inner'][0] + state['inner'][1]:
            buffer_insert(arm, state['root'], message)
    state['inner'] = [[], []]
    state['height'] = 2
    return finish_restructure(state, arm, before)


def op_rebalance(state, arm):
    """两棵子树之间搬边界：key b 从一个内部节点搬到另一个；rebalance_carry 时 b 在原节点缓冲里的消息随它搬走
    （变异 N1 / N2 不搬）。根缓冲管全部 key，不动。"""
    if state['height'] != 3:
        return []
    before = reads_snapshot(state, arm)
    source = route(state, 'b')
    state['pivot'] = 'c' if state['pivot'] == 'b' else 'b'
    target = route(state, 'b')
    if arm['rebalance_carry']:
        moving = [message for message in state['inner'][source] if message.key == 'b']
        state['inner'][source] = [message for message in state['inner'][source] if message.key != 'b']
        for message in moving:
            buffer_insert(arm, state['inner'][target], message)
    return finish_restructure(state, arm, before)


def strip_memory_fields(entry):
    return entry._replace(arrival=OLD, publish=OLD)


def snapshot_durable(state):
    return (state['height'], state['pivot'],
            tuple(strip_memory_fields(message) for message in state['root']),
            (tuple(strip_memory_fields(message) for message in state['inner'][0]),
             tuple(strip_memory_fields(message) for message in state['inner'][1])),
            tuple(strip_memory_fields(state['leaf'][key]) for key in sorted(state['leaf'], key=repr)))


def durable_view(durable):
    height, pivot, root, inner, leaf = durable
    return {'height': height, 'pivot': pivot, 'root': list(root), 'inner': [list(inner[0]), list(inner[1])],
            'leaf': {entry.key: entry for entry in leaf}}


def op_publish(state, arm):
    """发布点：按臂的 flush 目标把前端排空（丙 / 甲-R4 到叶，乙 到根缓冲），整棵树的内存镜像落盘（一次发布整体生效）。"""
    states = flush_front_end(state, arm) if state['front_end'] else [state]
    for current in states:
        if arm['check_second_writer'] and any(entry.origin != 'fe' and entry.publish == current['current_publish']
                                              for entry in current['leaf'].values()):
            current['second_writer_seen'] = True   # 检查 G3（丙）：这一窗口里前端树叶上的改动都得是从前端排空来的
        current['durable'] = snapshot_durable(current)
        current['truth_at_publish'] = dict(current['truth'])
        current['current_publish'] += 1
        current['seq_count_in_publish'] = {}
        current['durable_global_seq'] = current['global_seq']
    return states


def op_crash(state, arm):
    """崩溃：内存全丢，树回到上一次发布；发布号不动（没发出去的那个 txg 重发）。全池单调 seq 的计数器：
    persisted 从发布时落盘的那个值接着数；reset 从 0 重数（计数器不落盘时的最弱读法）。"""
    view = durable_view(state['durable'])
    state['height'], state['pivot'] = view['height'], view['pivot']
    state['root'], state['inner'], state['leaf'] = view['root'], view['inner'], view['leaf']
    state['front_end'] = {}
    state['seq_count_in_publish'] = {}
    state['truth'] = dict(state['truth_at_publish'])
    state['global_seq'] = state['durable_global_seq'] if arm['seq_on_crash'] == 'persisted' else 0
    for key in universe_of_keys(state):
        if read_tree(state, arm, key) != {state['truth'].get(key)}:
            state['recovered_wrong'] = True
    return [state]


def classify_reads(state, arm):
    """U1：每个 key 读一遍，与按时间序的真值比；另报 G1（重组改了读）、G2（缓冲里有不归这个节点管的 key）、G3（丙的第二个写者）。
    stale           读不到应得的值，路径上没用过盘上未定义的平局
    stale_after_tie 读不到应得的值，路径上用过一次盘上未定义的平局（取了对它最不利的那一边）
    undefined       读法本身给出两个答案（同 key 同 seq 而值不同）"""
    found = {}
    if state['recovered_wrong']:
        found['u2_recovered_content'] = None
    if state['restructure_changed_read']:
        found['g1_restructure_changed_read'] = None
    if state['second_writer_seen']:
        found['g3_second_writer'] = None
    if state['height'] == 3 and any(route(state, message.key) != index
                                    for index in (0, 1) for message in state['inner'][index]):
        found['g2_buffer_key_out_of_range'] = None
    for key in sorted(universe_of_keys(state), key=repr):
        outcomes = read_value(state, arm, key)
        expected = state['truth'].get(key)
        if expected not in outcomes:
            label = 'stale_after_tie' if state['tie'] else 'stale'
        elif len(outcomes) > 1:
            label = 'undefined'
        else:
            continue
        found.setdefault(label, key)
    return found


def detector_verdicts(arm, durable):
    """E：陈旧检出的检测器只能用盘上的东西。一个 key 在盘上有两条以上时，按线索挑「最新」那条，与臂按自己的规则读出的比：
    seq       只用 seq（第一版口径下是「本次发布里写进树的第几次」；全池单调时就是全局序）
    txg_seq   先比写进树的那次发布号、再比 seq（要在每条消息里多放一份发布号）
    返回 [(线索, 'tie' / 'agree' / 'disagree')]，盘上每个 key 只有一条时不出现（没东西要排序）。"""
    view = durable_view(durable)
    per_key = {}
    for entry in view['root'] + view['inner'][0] + view['inner'][1] + list(view['leaf'].values()):
        per_key.setdefault(entry.key, []).append(entry)
    verdicts = []
    for key in sorted(per_key, key=repr):
        entries = per_key[key]
        if len(entries) < 2:
            continue
        arm_outcomes = read_tree(view, arm, key)
        for clue_name in ('seq', 'txg_seq'):
            def clue(entry, clue_name=clue_name):
                return (entry.seq,) if clue_name == 'seq' else (entry.txg, entry.seq)
            top = max(clue(entry) for entry in entries)
            picked = {entry_outcome(entry) for entry in entries if clue(entry) == top}
            if len(picked) > 1:
                verdicts.append((clue_name, 'tie'))
            elif picked == arm_outcomes:
                verdicts.append((clue_name, 'agree'))
            else:
                verdicts.append((clue_name, 'disagree'))
    return verdicts


def canonical(state, arm):
    """状态签名：值按首次出现重新编号，到达序按秩重排，发布号与 txg 取相对值；全池单调且计数器落盘时 seq 按秩重排。只用于去重。"""
    current = state['current_publish']
    value_labels = {}

    def value_label(value):
        if value is None:
            return None
        if value not in value_labels:
            value_labels[value] = len(value_labels)
        return value_labels[value]

    def key_label(key):
        return ('s', key[1] - current) if isinstance(key, tuple) else key

    durable = state['durable']
    durable_entries = list(durable[2]) + list(durable[3][0]) + list(durable[3][1]) + list(durable[4])
    entries = (list(state['front_end'].values()) + state['root'] + state['inner'][0] + state['inner'][1]
               + list(state['leaf'].values()) + durable_entries)
    arrival_rank = {arrival: rank for rank, arrival in
                    enumerate(sorted({entry.arrival for entry in entries if entry.arrival != OLD}))}
    seq_rank = None
    if arm['seq_mode'] == 'global' and arm['seq_on_crash'] == 'persisted':
        pool = {entry.seq for entry in entries if entry.seq is not None}
        pool |= {state['global_seq'], state['durable_global_seq']}
        seq_rank = {seq: rank for rank, seq in enumerate(sorted(pool))}

    def seq_label(seq):
        return seq if seq is None or seq_rank is None else seq_rank[seq]

    def entry_label(entry):
        publish = 'old' if entry.publish == OLD else entry.publish - current
        txg = None if entry.txg is None else entry.txg - current
        return (repr(key_label(entry.key)), entry.kind, value_label(entry.value), seq_label(entry.seq),
                arrival_rank.get(entry.arrival, OLD), publish, txg, entry.origin)

    def ordered_keys(mapping):
        return sorted(mapping, key=lambda key: repr(key_label(key)))

    return (
        tuple((repr(key_label(key)), value_label(state['truth'][key])) for key in ordered_keys(state['truth'])),
        tuple((repr(key_label(key)), value_label(state['truth_at_publish'][key]))
              for key in ordered_keys(state['truth_at_publish'])),
        tuple(entry_label(state['front_end'][key]) for key in ordered_keys(state['front_end'])),
        state['height'], state['pivot'],
        tuple(entry_label(message) for message in state['root']),
        tuple(entry_label(message) for message in state['inner'][0]),
        tuple(entry_label(message) for message in state['inner'][1]),
        tuple(entry_label(state['leaf'][key]) for key in ordered_keys(state['leaf'])),
        (durable[0], durable[1], tuple(entry_label(message) for message in durable[2]),
         tuple(entry_label(message) for message in durable[3][0]),
         tuple(entry_label(message) for message in durable[3][1]),
         tuple(entry_label(entry) for entry in durable[4])),
        tuple(sorted((repr(key_label(key)), count) for key, count in state['seq_count_in_publish'].items())),
        seq_label(state['global_seq']), seq_label(state['durable_global_seq']),
        state['tie'], state['recovered_wrong'], state['restructure_changed_read'], state['second_writer_seen'],
    )


def breadth_first_search(arm, operations, depth_limit, classify, on_state=None):
    """按长度逐层穷举操作序列（状态签名去重），记下每类命中第一次出现的最短序列。on_state(state, path) 给 E 用。"""
    start = new_state()
    seen = {canonical(start, arm)}
    frontier = [(start, ())]
    first_hit = {}
    states_explored = 1
    for depth in range(depth_limit + 1):
        for state, path in frontier:
            if on_state is not None:
                on_state(state, path)
            for label, key in classify(state, arm).items():
                if label not in first_hit:
                    first_hit[label] = (depth, path, key)
        if depth == depth_limit:
            break
        next_frontier = []
        for state, path in frontier:
            for name, operation in operations:
                for successor in operation(copy_state(state), arm):
                    signature = canonical(successor, arm)
                    if signature not in seen:
                        seen.add(signature)
                        next_frontier.append((successor, path + (name,)))
        frontier = next_frontier
        states_explored += len(frontier)
    return first_hit, states_explored


def tree_operations(arm, restructure, rebalance):
    operations = [('flush', op_flush)]
    if arm['buffer_policy'] is not None:
        operations += [('push_root', lambda state, arm_: op_push_root(state, arm_)),
                       ('push_root_child0', lambda state, arm_: op_push_root(state, arm_, 0)),
                       ('push_root_child1', lambda state, arm_: op_push_root(state, arm_, 1)),
                       ('push_inner0', lambda state, arm_: op_push_inner(state, arm_, 0)),
                       ('push_inner1', lambda state, arm_: op_push_inner(state, arm_, 1))]
    if restructure:
        operations += [('grow', op_grow), ('shrink', op_shrink)]
        if rebalance:
            operations.append(('rebalance', op_rebalance))
    operations += [('publish', op_publish), ('crash', op_crash)]
    return operations


def generic_operations(arm, restructure=True):
    """通用负载：两个不带代的 key（反向索引 / LRU 的 key 布局仓里没定，按不带代算）。"""
    operations = [('fe_put_a', lambda state, arm_: front_end_update(state, arm_, 'a', 'P')),
                  ('fe_put_b', lambda state, arm_: front_end_update(state, arm_, 'b', 'P')),
                  ('fe_del_b', lambda state, arm_: front_end_update(state, arm_, 'b', 'D'))]
    if arm['direct_target'] != 'front_end':
        operations.append(('direct_put_b', lambda state, arm_: direct_update(state, arm_, 'b', 'P')))
    return operations + tree_operations(arm, restructure, rebalance=True)


def accounting_operations(arm, retained):
    """记账负载：一个统计量、key 带代、只保留最近 K = retained 代，丢弃是点删（当前代 − K）。"""
    def discard(via_front_end):
        def operation(state, arm_):
            key = ('s', state['current_publish'] - retained)
            if key[1] < 0 or key not in state['truth']:
                return []
            if via_front_end:
                return front_end_update(state, arm_, key, 'D')
            return direct_update(state, arm_, key, 'D')
        return operation

    operations = [('acc_put', lambda state, arm_: front_end_update(state, arm_, ('s', state['current_publish']), 'P')),
                  ('acc_discard_fe', discard(True))]
    if arm['direct_target'] != 'front_end':
        operations.append(('acc_discard_direct', discard(False)))
    return operations + tree_operations(arm, restructure=True, rebalance=False)


BASE_ARM = dict(direct_target='front_end', seq_mode='per_publish', seq_assign='flush', seq_on_crash='persisted',
                collapse='push_down', rebalance_carry=True, purge_route='current', direct_evicts_front_end=False,
                check_second_writer=False)
ARM_DEFINITIONS = {
    # 甲-R4：前端 flush 直落叶（清路径上比它旧的同 key 消息、落叶比同一窗口的到达序）；不走前端的写进根缓冲并逐出前端
    'jia_R4': dict(flush_target='leaf_purge', direct_target='root', buffer_policy='append', direct_evicts_front_end=True),
    # 乙-D：前端 flush 进根缓冲、逐层下推；一个缓冲区里同 key 至多一条，新来的替换旧的；跨层、同层都按位置
    'yi_D': dict(flush_target='root', buffer_policy='dedup'),
    # 乙 + seq 全池单调：乙字面（跨层看位置、同一层由 seq 定），seq 全池单调、到达前端即取号、计数器随发布落盘
    'yi_G': dict(flush_target='root', buffer_policy='seq', seq_mode='global', seq_assign='arrival'),
    # 丙：前端树 ε = 0、前端树上的写一律进前端；发布前排空到叶，落叶比同一窗口的到达序
    'bing': dict(flush_target='leaf_arrival_guard', buffer_policy=None, check_second_writer=True),
    # 阳性对照：乙字面 + 第一版 seq 口径（第一轮已打中）
    'yi_S': dict(flush_target='root', buffer_policy='seq'),
}


def make_arm(name, base=None, **overrides):
    arm = dict(BASE_ARM)
    arm.update(ARM_DEFINITIONS[base or name])
    arm.update(overrides)
    arm['name'] = name
    return arm


# ---------------------------------------------------------------- C：fsync 的部分发布与发布集的边界
# 三棵树：extent 树（ext，不走前端）、反向索引树（ri，走前端）、记账树（acc，走前端或发布时现算）。
# 节点：root（根缓冲，只有带缓冲的树有）、LA / LB（A、B 两个文件的条目各住一片叶）或 LS（同一片叶）、LACC（记账叶）。
PublishedEntry = namedtuple('PublishedEntry', 'value owner')   # owner：写它的文件 'A' / 'B'；发布时算出的记账行 'acc'
PC_TREES = ('ext', 'ri', 'acc')
PC_LEAVES = ('LA', 'LB', 'LS', 'LACC')


def pc_has_root_buffer(config, tree):
    """哪棵树的根带缓冲：extent 树看 buffered_ext（ε > 0 那一版按树轴留缓冲，与臂无关）；
    前端树看臂——乙两棵都留，甲-R4 的反向索引树留（收不走前端的写），丙都不留。"""
    if tree == 'ext':
        return config['buffered_ext']
    if tree == 'ri':
        return config['arm'] in ('yi_D', 'jia_R4')
    return config['arm'] == 'yi_D'


def pc_leaf(config, tree, owner):
    if tree == 'acc':
        return 'LACC'
    if config['shared_leaf']:
        return 'LS'
    return 'LA' if owner == 'A' else 'LB'


def pc_new_state():
    return {'mem': {tree: {} for tree in PC_TREES}, 'dur': {tree: {} for tree in PC_TREES}, 'front_end': {},
            'extent_counter': 0, 'owners': {}, 'generation': 0,
            'fsynced': frozenset(), 'ever_durable': frozenset(), 'labels': frozenset()}


def pc_copy_trees(trees):
    return {tree: {node: dict(content) for node, content in nodes.items()} for tree, nodes in trees.items()}


def pc_copy(state):
    copied = dict(state)
    copied['mem'] = pc_copy_trees(state['mem'])
    copied['dur'] = pc_copy_trees(state['dur'])
    copied['front_end'] = dict(state['front_end'])
    copied['owners'] = dict(state['owners'])
    return copied


def pc_put(trees, tree, node, key, entry):
    trees[tree].setdefault(node, {})[key] = entry


def pc_write(state, config, owner):
    """一次写：分配一个 extent（extent 树直接改）；反向索引条目进前端（甲-R4 的第二个写者直接进反向索引树的根缓冲）；
    记账 write_time：写入那一刻就把「已分配」的完整值算好、按当前代进前端。"""
    state['extent_counter'] += 1
    extent = state['extent_counter']
    state['owners'][extent] = owner
    ext_node = 'root' if config['buffered_ext'] else pc_leaf(config, 'ext', owner)
    pc_put(state['mem'], 'ext', ext_node, ('ext', extent), PublishedEntry(1, owner))
    if config['arm'] == 'jia_R4' and owner == 'B' and config['b_direct']:
        pc_put(state['mem'], 'ri', 'root', ('ri', extent), PublishedEntry(1, owner))
    else:
        state['front_end'][('ri', extent)] = PublishedEntry(1, owner)
    if config['accounting'] == 'write_time':
        state['front_end'][('alloc', state['generation'])] = PublishedEntry(len(state['owners']), owner)
    return [state]


def pc_drain(state, config, only_owner=None):
    """前端进树：乙进根缓冲，丙 / 甲-R4 落叶。only_owner 给了就只排空那个文件的条目，别的留在前端（按事务留住）。"""
    kept = {}
    for key in sorted(state['front_end'], key=repr):
        entry = state['front_end'][key]
        if only_owner is not None and entry.owner != only_owner:
            kept[key] = entry
            continue
        tree = 'acc' if key[0] == 'alloc' else 'ri'
        node = 'root' if config['arm'] == 'yi_D' else pc_leaf(config, tree, entry.owner)
        pc_put(state['mem'], tree, node, key, entry)
    state['front_end'] = kept


def pc_flush(state, config):
    if not state['front_end']:
        return []
    pc_drain(state, config)
    return [state]


def pc_push(state, config):
    """带缓冲的树把根缓冲整份推到叶（只在内存里）。"""
    moved = False
    for tree in PC_TREES:
        root = state['mem'][tree].get('root')
        if not root:
            continue
        for key, entry in root.items():
            pc_put(state['mem'], tree, pc_leaf(config, tree, entry.owner), key, entry)
        state['mem'][tree]['root'] = {}
        moved = True
    return [state] if moved else []


def pc_view(trees):
    """每棵树读出来的 key → 条目：根缓冲里的比叶里的新（位置即新旧）。"""
    view = {}
    for tree in PC_TREES:
        merged = {}
        for node in PC_LEAVES:
            merged.update(trees[tree].get(node, {}))
        merged.update(trees[tree].get('root', {}))
        view[tree] = merged
    return view


def pc_durable_keys(state):
    view = pc_view(state['dur'])
    return frozenset((tree, key) for tree in PC_TREES for key in view[tree])


def pc_add_accounting_row(state, config, trees, value):
    node = 'root' if config['arm'] == 'yi_D' else 'LACC'
    pc_put(trees, 'acc', node, ('alloc', state['generation']), PublishedEntry(value, 'acc'))


def pc_finish_publish(state, is_fsync):
    if is_fsync:
        state['fsynced'] = state['fsynced'] | frozenset(extent for extent, owner in state['owners'].items()
                                                        if owner == 'A')
    state['generation'] += 1          # 每次发布 checkpoint_txg + 1，fsync 触发的也是（D16 已定项 6）
    state['ever_durable'] = state['ever_durable'] | pc_durable_keys(state)
    return [state]


def pc_publish_full(state, config, is_fsync=False):
    """全量发布：前端全部排空，内存里的树整份落盘。budget 给了只排空前 budget 条，其余留到下一次（排空跨两次发布）。"""
    if config['budget'] is None:
        pc_drain(state, config)
    else:
        chosen = set(sorted(state['front_end'], key=repr)[:config['budget']])
        held = {key: entry for key, entry in state['front_end'].items() if key not in chosen}
        state['front_end'] = {key: entry for key, entry in state['front_end'].items() if key in chosen}
        pc_drain(state, config)
        state['front_end'] = held
    if config['accounting'] == 'publish_time':
        pc_add_accounting_row(state, config, state['mem'], len(state['owners']))
    state['dur'] = pc_copy_trees(state['mem'])
    return pc_finish_publish(state, is_fsync)


def pc_fsync(state, config):
    """fsync 文件 A。partial_publish：
    full            fsync 就是提前触发一次全量发布（D16:22「fsync 的效果是『提前触发一次发布』」的字面读法）
    path_in_memory  部分发布：A 的路径（每棵树的根缓冲 + A 的叶 + 记账叶）按内存里的样子写出去；ε = 0 的内部节点只有指针，
                    按「没写的孩子指旧版本」算（对这条规则最有利的读法）
    filtered        部分发布：写出去的镜像 = 盘上原有的 + 发布集（A 的事务）的改动，B 的改动留在内存
    drain：all 前端按时间窗口全部排空；hold_back 只排空 A 的条目（按事务留住 B 的）。"""
    rule = config['partial_publish']
    if rule == 'full':
        return pc_publish_full(state, config, is_fsync=True)
    pc_drain(state, config, 'A' if config['drain'] == 'hold_back' else None)
    if rule == 'path_in_memory':
        for tree in ('ext', 'ri'):
            nodes = (['root'] if pc_has_root_buffer(config, tree) else []) + [pc_leaf(config, tree, 'A')]
            for node in nodes:
                state['dur'][tree][node] = dict(state['mem'][tree].get(node, {}))
        if config['accounting'] == 'publish_time':
            # 按这次实际写出去的算：新盘面上的 extent 数（同样是对这条规则最有利的读法）
            pc_add_accounting_row(state, config, state['mem'], len(pc_view(state['dur'])['ext']))
        for node in (['root'] if pc_has_root_buffer(config, 'acc') else []) + ['LACC']:
            state['dur']['acc'][node] = dict(state['mem']['acc'].get(node, {}))
        return pc_finish_publish(state, is_fsync=True)
    if rule == 'filtered':
        for tree in PC_TREES:
            durable_keys = set(pc_view(state['dur'])[tree])
            for node, content in state['mem'][tree].items():
                for key, entry in content.items():
                    if entry.owner == 'A' and key not in durable_keys:
                        pc_put(state['dur'], tree, node, key, entry)
        if config['accounting'] == 'publish_time':
            value = len(pc_view(state['dur'])['ext'])
            pc_add_accounting_row(state, config, state['mem'], value)
            pc_add_accounting_row(state, config, state['dur'], value)
        return pc_finish_publish(state, is_fsync=True)
    raise ValueError(rule)


def pc_crash(state, config):
    """崩溃：按盘上镜像读出三棵树，查四样；内存回到盘上、前端全丢，发布号不动（没发出去的那个 txg 重发）。
    I-3.1            最新一代的「已分配」 == 盘上 extent 数
    reverse_index    反向索引条目集 == extent 集
    fsync_lost       fsync 返回过的 A 的 extent 不在盘上
    published_lost   某次发布之后盘上读得到的条目，现在读不到了（模型里没有删除，读不到就是丢了）"""
    view = pc_view(state['dur'])
    extents = {key[1] for key in view['ext']}
    reverse = {key[1] for key in view['ri']}
    rows = {key[1]: entry.value for key, entry in view['acc'].items()}
    allocated = rows[max(rows)] if rows else 0
    labels = set()
    if allocated != len(extents):
        labels.add('I-3.1')
    if reverse != extents:
        labels.add('reverse_index')
    if not state['fsynced'] <= extents:
        labels.add('fsync_lost')
    if not state['ever_durable'] <= pc_durable_keys(state):
        labels.add('published_lost')
    state['labels'] = state['labels'] | frozenset(labels)
    state['mem'] = pc_copy_trees(state['dur'])
    state['front_end'] = {}
    state['owners'] = {key[1]: entry.owner for key, entry in view['ext'].items()}
    return [state]


def pc_operations(config):
    operations = [('write_A', lambda state: pc_write(state, config, 'A')),
                  ('write_B', lambda state: pc_write(state, config, 'B'))]
    if config['flush_between']:        # 两次发布之间前端满了就 flush（不等发布点）
        operations.append(('flush', lambda state: pc_flush(state, config)))
        if any(pc_has_root_buffer(config, tree) for tree in PC_TREES):
            operations.append(('push', lambda state: pc_push(state, config)))
    operations += [('fsync_A', lambda state: pc_fsync(state, config)),
                   ('publish', lambda state: pc_publish_full(state, config)),
                   ('crash', lambda state: pc_crash(state, config))]
    return operations


def pc_canonical(state):
    generation = state['generation']

    def key_label(key):
        return (key[0], key[1] - generation) if key[0] == 'alloc' else key

    def trees_label(trees):
        return tuple((tree, tuple((node, tuple(sorted((repr(key_label(key)), entry) for key, entry in content.items())))
                                  for node, content in sorted(trees[tree].items()) if content))
                     for tree in PC_TREES)

    return (trees_label(state['mem']), trees_label(state['dur']),
            tuple(sorted((repr(key_label(key)), entry) for key, entry in state['front_end'].items())),
            state['extent_counter'], tuple(sorted(state['owners'].items())), tuple(sorted(state['fsynced'])),
            tuple(sorted(repr((tree, key_label(key))) for tree, key in state['ever_durable'])), state['labels'])


def pc_search(config, depth_limit):
    start = pc_new_state()
    seen = {pc_canonical(start)}
    frontier = [(start, ())]
    first_hit = {}
    explored = 1
    operations = pc_operations(config)
    for depth in range(depth_limit + 1):
        for state, path in frontier:
            for label in sorted(state['labels']):
                if label not in first_hit:
                    first_hit[label] = (depth, path)
        if depth == depth_limit:
            break
        next_frontier = []
        for state, path in frontier:
            for name, operation in operations:
                for successor in operation(pc_copy(state)):
                    signature = pc_canonical(successor)
                    if signature not in seen:
                        seen.add(signature)
                        next_frontier.append((successor, path + (name,)))
        frontier = next_frontier
        explored += len(frontier)
    return first_hit, explored


PC_ARMS = (('bing', False), ('yi_D', False), ('jia_R4', False), ('jia_R4', True))   # (臂, B 的反向索引写走不走前端)


def pc_configs():
    """C 的格子：extent 树带不带缓冲 × A、B 的条目同不同叶 × 两次发布之间前端会不会 flush × 记账怎么算 ×
    部分发布的规则与排空方式 × 臂。"""
    configs = []
    for buffered_ext in (False, True):
        for shared_leaf in (False, True):
            for flush_between in (False, True):
                for accounting in ('publish_time', 'write_time'):
                    for partial_publish, drain in (('full', 'all'), ('path_in_memory', 'all'),
                                                   ('path_in_memory', 'hold_back'), ('filtered', 'all'),
                                                   ('filtered', 'hold_back')):
                        for arm, b_direct in PC_ARMS:
                            configs.append(dict(arm=arm, b_direct=b_direct, partial_publish=partial_publish,
                                                drain=drain, accounting=accounting, flush_between=flush_between,
                                                buffered_ext=buffered_ext, shared_leaf=shared_leaf, budget=None))
    return configs


def pc_budget_configs():
    """D：排空跨两次发布——全量发布只排空前 1 条前端条目，树照常整份落盘。"""
    return [dict(arm=arm, b_direct=False, partial_publish='full', drain='all', accounting=accounting,
                 flush_between=False, buffered_ext=False, shared_leaf=False, budget=1)
            for arm in ('bing', 'yi_D', 'jia_R4') for accounting in ('publish_time', 'write_time')]


# ---------------------------------------------------------------- F：U3 的两笔账
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
    """（抄自第一轮模型）丙 / 甲 的前端树（ε = 0）：每次发布把前端排空到叶，写出的节点 = 碰到的叶 + 全部祖先（一次发布里去重）。"""
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
    """（抄自第一轮模型）乙（ε > 0）：每次发布把前端排空进根缓冲；任一内部节点的缓冲超过容量就把消息最多的那个孩子整份下推（级联）。
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


def make_leaf_sampler(workload, leaf_count, generator):
    if workload == 'uniform':
        return lambda: generator.randrange(leaf_count)
    if workload == 'zipf':                        # Zipf(1)，热叶随机散在各棵子树里
        cumulative = list(itertools.accumulate(1.0 / (rank + 1) for rank in range(leaf_count)))
        permutation = list(range(leaf_count))
        random.Random(7).shuffle(permutation)
        total = cumulative[-1]
        return lambda: permutation[bisect.bisect_left(cumulative, generator.random() * total)]
    if workload == 'hot_cold':                    # 99% 的写落在连续的 1% 叶上，其余 1% 均匀撒在全部叶上
        hot = max(1, leaf_count // 100)
        return lambda: generator.randrange(hot) if generator.random() < 0.99 else generator.randrange(leaf_count)
    raise ValueError(workload)


def simulate_residence(leaf_count, fanout, capacity, workload, message_count, seed):
    """乙 + 全池单调：消息进根缓冲时取 seq（= 全池第几次写），缓冲超过容量就把消息最多的那个孩子整份下推（第一轮同一条策略）。
    量三样：落叶时离取号过了多少次写（停留期上界：同 key 两条在同一个缓冲里相遇时 seq 之差不超过它）；
    跑完时还困在缓冲里的最老消息的年龄（右截断）；每条消息摊到的下推次数。"""
    generator = random.Random(seed)
    sampler = make_leaf_sampler(workload, leaf_count, generator)
    height = tree_height(leaf_count, fanout)
    divisor = [fanout ** level for level in range(height)]
    buffers, counts = {}, {}
    stats = {'max_residence': 0, 'pushes': 0}

    def cascade(node, now):
        level = node[0]
        group_map = buffers[node]
        while counts[node] > capacity:
            child = max(group_map, key=lambda index: (len(group_map[index]), -index))
            batch = group_map.pop(child)
            counts[node] -= len(batch)
            stats['pushes'] += 1
            if level == 1:                        # 落叶：一个组里的消息按 seq 排着，第一条最老
                stats['max_residence'] = max(stats['max_residence'], now - batch[0][1])
                continue
            child_node = (level - 1, child)
            child_map = buffers.setdefault(child_node, {})
            for leaf, seq in batch:
                child_map.setdefault(leaf // divisor[level - 2], []).append((leaf, seq))
            counts[child_node] = counts.get(child_node, 0) + len(batch)
            if counts[child_node] > capacity:
                cascade(child_node, now)

    root = (height - 1, 0)
    root_map = buffers.setdefault(root, {})
    counts[root] = 0
    for seq in range(1, message_count + 1):
        leaf = sampler()
        root_map.setdefault(leaf // divisor[height - 2], []).append((leaf, seq))
        counts[root] += 1
        if counts[root] > capacity:
            cascade(root, seq)
    oldest = min((group[0][1] for group_map in buffers.values() for group in group_map.values()),
                 default=message_count)
    return {'height': height, 'max_residence': stats['max_residence'], 'oldest_resident_age': message_count - oldest,
            'resident': sum(counts.values()), 'pushes_per_message': stats['pushes'] / message_count}


# ---------------------------------------------------------------- 主程序
U1_LABELS = ('stale', 'undefined', 'stale_after_tie', 'u2_recovered_content', 'g1_restructure_changed_read',
             'g2_buffer_key_out_of_range', 'g3_second_writer')
PC_LABELS = ('I-3.1', 'reverse_index', 'fsync_lost', 'published_lost')


def part_a_arms():
    """收严形态 + 阳性对照 + 两处定义空白的最弱读法（塌缩按 fold_up、全池单调的计数器不落盘）。"""
    return [make_arm('jia_R4'), make_arm('yi_D'), make_arm('yi_G'), make_arm('bing'), make_arm('yi_S'),
            make_arm('jia_R4_fold_up', 'jia_R4', collapse='fold_up'),
            make_arm('yi_D_fold_up', 'yi_D', collapse='fold_up'),
            make_arm('yi_G_fold_up', 'yi_G', collapse='fold_up'),
            make_arm('yi_G_counter_reset', 'yi_G', seq_on_crash='reset')]


def part_b_arms():
    return [make_arm('jia_R4'), make_arm('yi_D'), make_arm('yi_G'), make_arm('bing'), make_arm('yi_S')]


MUTATIONS = [
    # (编号, 被改坏的臂, 改坏了什么, 说明)：把收严形态改坏一处，同一套检查（U1 读对 + G1 / G2 / G3）必须抓到
    ('N1', 'yi_D', dict(rebalance_carry=False), '乙-D：rebalance 不搬缓冲里的消息'),
    ('N2', 'jia_R4', dict(rebalance_carry=False), '甲-R4：rebalance 不搬缓冲里的消息'),
    ('N3', 'jia_R4', dict(purge_route='stale'), '甲-R4：flush 清路径时走错子树'),
    ('N4', 'bing', dict(direct_target='leaf_blind'), '丙：前端树上多一个直落叶的写者'),
    ('N5', 'yi_G', dict(seq_on_crash='reset'), '乙 + 全池单调：计数器不落盘，掉电后从 0 数'),
    ('N6', 'yi_D', dict(collapse='fold_up'), '乙-D：塌缩时孩子的消息当新来的放进根缓冲'),
]


def format_path(path):
    return f"{len(path)}:{','.join(path) if path else '(empty)'}"


def format_hit(first_hit, label):
    if label not in first_hit:
        return '-'
    depth, path, key = first_hit[label]
    where = '' if key is None else '|READ(' + repr(key).replace(' ', '') + ')'
    return f"{depth}:{','.join(path) if path else '(empty)'}{where}"


def detector_accumulator(arm):
    """E：BFS 走到的每一份盘上镜像（按原值去重）跑一遍检测器，数 agree / tie / disagree，记第一次出现 tie、disagree 的序列。"""
    tally, first_path, seen_durable = {}, {}, set()

    def on_state(state, path):
        durable = state['durable']
        if durable in seen_durable:
            return
        seen_durable.add(durable)
        verdicts = detector_verdicts(arm, durable)
        if verdicts:
            tally['with_duplicate_keys'] = tally.get('with_duplicate_keys', 0) + 1
        for clue_verdict in verdicts:
            tally[clue_verdict] = tally.get(clue_verdict, 0) + 1
            if clue_verdict[1] != 'agree' and clue_verdict not in first_path:
                first_path[clue_verdict] = path
    return on_state, tally, first_path, seen_durable


def detector_line(workload, arm, tally, first_path, seen_durable):
    parts = [f"C161R2 name=E_detector workload={workload} arm={arm['name']} durable_images={len(seen_durable)}",
             f"with_duplicate_keys={tally.get('with_duplicate_keys', 0)}"]
    for clue_name in ('seq', 'txg_seq'):
        parts.append(clue_name + ':' + ','.join(f"{verdict}={tally.get((clue_name, verdict), 0)}"
                                               for verdict in ('agree', 'tie', 'disagree')))
    for clue_name in ('seq', 'txg_seq'):
        for verdict in ('tie', 'disagree'):
            path = first_path.get((clue_name, verdict))
            parts.append(f"first_{clue_name}_{verdict}={'-' if path is None else format_path(path)}")
    return ' '.join(parts)


def run_u1(emit, workload, arm, operations, depth_limit):
    on_state, tally, first_path, seen_durable = detector_accumulator(arm)
    first_hit, states = breadth_first_search(arm, operations, depth_limit, classify_reads, on_state)
    hits = ' '.join(f"{label}={format_hit(first_hit, label)}" for label in U1_LABELS)
    emit(f"C161R2 name=A_u1 workload={workload} arm={arm['name']} depth_limit={depth_limit} states={states} {hits}")
    emit(detector_line(workload, arm, tally, first_path, seen_durable))


def write_volume_rows(leaf_count, seed):
    """每次 fsync 摊到的写出节点数（今天的宽：pivot 117、消息 34）。own = 被 fsync 的文件这次的前端条目数，background = 两次 fsync
    之间别的文件进前端的条目数，window = 两次全量发布之间的 fsync 次数。R-full：每次 fsync 都是全量发布，排空 own + background 条；
    R-partial：每次 fsync 只排空 own 条（按事务留住别的），每 window 次 fsync 一次全量发布排空 window × background 条。
    丙 / 甲-R4 排空到叶（ε = 0），乙排空到根缓冲（ε = 0.65）。"""
    fanout_plain, fanout_buffered = fanout_for(0.0, 117), fanout_for(0.65, 117)
    capacity = buffer_messages_for(0.65, 34)
    cache = {}

    def cost(kind, entries, publishes):
        if entries == 0:
            return 0.0
        if (kind, entries) not in cache:
            if kind == 'leaf':
                cache[(kind, entries)] = nodes_written_drain_to_leaf(leaf_count, fanout_plain, entries, publishes, seed)[0]
            else:
                cache[(kind, entries)] = nodes_written_drain_to_root_buffer(leaf_count, fanout_buffered, capacity,
                                                                            entries, publishes, seed)[0]
        return cache[(kind, entries)]

    rows = []
    for own, background, window in ((1, 0, 100), (1, 4, 100), (1, 64, 100), (3, 16, 100)):
        full_leaf, full_buffer = cost('leaf', own + background, 400), cost('buffer', own + background, 400)
        partial_leaf = cost('leaf', own, 400) + cost('leaf', window * background, 20) / window
        partial_buffer = cost('buffer', own, 400) + cost('buffer', window * background, 20) / window
        rows.append((own, background, window, full_leaf, full_buffer, partial_leaf, partial_buffer))
    return rows


def main():
    depth_generic = int(sys.argv[1]) if len(sys.argv) > 1 else 9
    depth_accounting = int(sys.argv[2]) if len(sys.argv) > 2 else 11
    depth_partial = int(sys.argv[3]) if len(sys.argv) > 3 else 6
    residence_messages = int(sys.argv[4]) if len(sys.argv) > 4 else 10_000_000
    lines = []
    emit = lines.append
    # A：两棵子树 + 分裂合并 + 部分下推（通用负载）；E 的检测器跟着同一次穷举跑
    for arm in part_a_arms():
        # 计数器不落盘那一格的状态签名不做 seq 重排、状态数长得快；它第 5–6 步就中，深度减一
        depth = depth_generic - 1 if arm['seq_on_crash'] == 'reset' else depth_generic
        run_u1(emit, 'generic', arm, generic_operations(arm), depth)
    # B：K > 1 的记账点删
    for retained in (2, 3):
        for arm in part_b_arms():
            run_u1(emit, f'accounting_K{retained}', arm, accounting_operations(arm, retained), depth_accounting)
    # U4：把收严形态改坏一处，同一套检查必须抓到
    mutation_depth = min(depth_generic, 8)
    for number, base, overrides, description in MUTATIONS:
        arm = make_arm(f"{base}+{number}", base, **overrides)
        first_hit, states = breadth_first_search(arm, generic_operations(arm), mutation_depth, classify_reads)
        found = [label for label in U1_LABELS if label in first_hit]
        shortest = format_hit(first_hit, min(found, key=lambda label: first_hit[label][0])) if found else '-'
        emit(f"C161R2 name=U4_mutation id={number} arm={base} depth_limit={mutation_depth} states={states} "
             f"caught={bool(found)} labels={'+'.join(found) if found else '-'} shortest={shortest} "
             f"what={description.replace(' ', '_')}")
    # C / D：fsync 的部分发布与发布集的边界；排空跨两次发布
    for config in pc_configs() + pc_budget_configs():
        first_hit, explored = pc_search(config, depth_partial)
        hits = ' '.join(f"{label}={first_hit[label][0]}:{','.join(first_hit[label][1])}" if label in first_hit
                        else f"{label}=-" for label in PC_LABELS)
        arm_name = config['arm'] + ('_B_direct' if config['b_direct'] else '')
        emit(f"C161R2 name=C_partial_publish arm={arm_name} fsync={config['partial_publish']} drain={config['drain']} "
             f"accounting={config['accounting']} flush_between={config['flush_between']} "
             f"buffered_ext={config['buffered_ext']} shared_leaf={config['shared_leaf']} budget={config['budget']} "
             f"depth_limit={depth_partial} states={explored} {hits}")
    # F：U3——全池单调 seq 下的停留期；部分发布下排空到叶的写出量；seq 宽度的时间换算
    fanout_buffered, capacity = fanout_for(0.65, 117), buffer_messages_for(0.65, 34)
    for leaf_count in (1024, 32768):
        for workload in ('uniform', 'zipf', 'hot_cold'):
            for message_count in (residence_messages // 10, residence_messages * 3 // 10, residence_messages):
                result = simulate_residence(leaf_count, fanout_buffered, capacity, workload, message_count, seed=161)
                emit(f"C161R2 name=F_residence leaves={leaf_count} F={fanout_buffered} B={capacity} "
                     f"h={result['height']} workload={workload} messages={message_count} "
                     f"max_residence={result['max_residence']} oldest_resident_age={result['oldest_resident_age']} "
                     f"resident={result['resident']} pushes_per_message={result['pushes_per_message']:.4f}")
    for own, background, window, full_leaf, full_buffer, partial_leaf, partial_buffer in write_volume_rows(32768, 161):
        emit(f"C161R2 name=F_write_volume leaves=32768 own={own} background={background} window={window} "
             f"full:leaf={full_leaf:.3f},root_buffer={full_buffer:.3f},ratio={full_leaf / full_buffer:.3f} "
             f"partial:leaf={partial_leaf:.3f},root_buffer={partial_buffer:.3f},ratio={partial_leaf / partial_buffer:.3f}")
    for rate in (10_000, 100_000, 750_000, 1_000_000):
        emit(f"C161R2 name=F_seq_width entries_per_second={rate} serial_window_2^31_seconds={2 ** 31 / rate:.0f} "
             f"plain_wrap_2^32_seconds={2 ** 32 / rate:.0f} window_2^63_years={2 ** 63 / rate / 31_557_600:.3e}")
    lines.append(f"emitted={len(lines)}")
    print('\n'.join(lines))


if __name__ == '__main__':
    main()
