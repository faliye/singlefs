#!/usr/bin/env python3
"""C161（缓冲两级并存而衔接没定） 第三轮攻方腿 · C 部分：单调统计量的点删——「取最大值」遇到墓碑。

一个单调统计量（D5 已定项 3 第 2 条那一类，今天是 inode 号水位、清扫水位、最近一次根销毁的代号），
key = (统计量, 代)；每次发布写这一代的完整值（水位），并点删 K 代之前的那一代（D5 已定项 2 的点删），K = 2。
树：根缓冲 → 一片叶（丙不用根缓冲）。臂：丙、甲-R4（点删走前端 / 走不走前端的写两种）、乙-D、乙-G。
两个读法开关：「取最大值」用在哪——buffer：只在缓冲里的合并（乙-D 定义里写的「单调统计量取最大」、乙-G 同层）；
everywhere：落叶也按统计量的合并规则合并。墓碑怎么比——wins：墓碑恒胜；loses：墓碑没有值，取最大时恒输；
order：退回臂自己的新旧规则（乙-G 看 seq，其余看到达）。判据：每个状态把每一代读一遍比真值，被点删的那一代应当读不到。"""
import sys

K = 2
GENERATIONS = 6
BUFFER_CAPACITY = 3
MAX_WATERMARK = 3
# 状态：(根缓冲, 叶, 水位, 下一代, 计数器, 盘上快照, 真值)；消息 (代, 值或 None=墓碑, seq)；叶 ((代, 值, seq), ...)


def combine(arm, context, existing, incoming):
    """同一个 key 的两条相遇，返回留下的那一条（(值或 None, seq)）；context 是 'buffer' 或 'leaf'。"""
    if not (arm['where_max'] == 'everywhere' or context == 'buffer'):
        return incoming                                    # 落叶按到达序：后到的替换
    (old_value, old_seq), (new_value, new_seq) = existing, incoming
    if old_value is not None and new_value is not None:
        return existing if old_value > new_value else incoming
    if old_value is None and new_value is None:
        return incoming
    if arm['tomb'] == 'wins':
        return existing if old_value is None else incoming
    if arm['tomb'] == 'loses':
        return existing if old_value is not None else incoming
    if arm['kind'] == 'yiG':
        return existing if old_seq > new_seq else incoming
    return incoming


def fold_group(arm, context, messages):
    result = None
    for message in sorted(messages, key=lambda item: item[2]) if arm['kind'] == 'yiG' else messages:
        pair = (message[1], message[2])
        result = pair if result is None else combine(arm, context, result, pair)
    return result


def apply_to_leaf(arm, leaf, messages):
    items = {gen: (value, seq) for gen, value, seq in leaf}
    groups = {}
    for message in messages:
        groups.setdefault(message[0], []).append(message)
    for gen, group in groups.items():
        incoming = fold_group(arm, 'buffer', group) if arm['kind'] == 'yiG' else None
        for pair in [incoming] if incoming is not None else [(m[1], m[2]) for m in group]:
            merged = pair if gen not in items else combine(arm, 'leaf', items[gen], pair)
            if merged[0] is None:
                items.pop(gen, None)
            else:
                items[gen] = merged
    return tuple(sorted((gen, value, seq) for gen, (value, seq) in items.items()))


def insert_buffer(arm, buffer, message):
    if arm['kind'] == 'yiD':
        same = [m for m in buffer if m[0] == message[0]]
        others = [m for m in buffer if m[0] != message[0]]
        if same:
            kept = combine(arm, 'buffer', (same[0][1], same[0][2]), (message[1], message[2]))
            return tuple(others) + ((message[0],) + kept,)
    return tuple(buffer) + (message,)


def read_gen(arm, state, gen):
    buffer, leaf = state[0], state[1]
    group = [m for m in buffer if m[0] == gen]
    if group:
        return (fold_group(arm, 'buffer', group) if arm['kind'] == 'yiG' else (group[-1][1], group[-1][2]))[0]
    return dict((item[0], item[1]) for item in leaf).get(gen)


def op_alloc(arm, state):
    return None if state[2] >= MAX_WATERMARK else (state[0], state[1], state[2] + 1) + state[3:]


def op_push(arm, state):
    if not state[0]:
        return None
    return ((), apply_to_leaf(arm, state[1], state[0])) + state[2:]


def op_publish(arm, state):
    buffer, leaf, watermark, gen, counter, _, truth = state
    if gen >= GENERATIONS:
        return None
    entries = [(gen, watermark)] + ([(gen - K, None)] if gen >= K else [])
    for key, value in entries:
        counter += 1
        message = (key, value, counter)
        if arm['kind'] in ('bing', 'jia') and not (value is None and arm.get('discard') == 'direct'):
            buffer = tuple(m for m in buffer if m[0] != key) if arm['kind'] == 'jia' else buffer   # 甲-R4 清路径
            leaf = apply_to_leaf(arm, leaf, [message])
        else:                                                  # 乙进根缓冲；甲走不走前端的写的点删也进根缓冲
            buffer = insert_buffer(arm, buffer, message)
    if len(buffer) > BUFFER_CAPACITY:
        buffer, leaf = (), apply_to_leaf(arm, leaf, buffer)
    truth = list(truth)
    truth[gen] = watermark
    if gen >= K:
        truth[gen - K] = None
    snapshot = (buffer, leaf, watermark, gen + 1, counter)
    return (buffer, leaf, watermark, gen + 1, counter, snapshot, tuple(truth))


def op_crash(arm, state):
    return state[5] + (state[5], state[6])


def search(arm, depth_limit):
    start_snapshot = ((), (), 0, 0, 0)
    start = start_snapshot + (start_snapshot, (None,) * GENERATIONS)
    parents, frontier, first_hit = {start: None}, [start], None
    for _ in range(depth_limit):
        next_frontier = []
        for state in frontier:
            for label, operation in (('alloc', op_alloc), ('publish', op_publish), ('push', op_push), ('crash', op_crash)):
                successor = operation(arm, state)
                if successor is None or successor in parents:
                    continue
                parents[successor] = (state, label)
                next_frontier.append(successor)
                if first_hit is None and any(read_gen(arm, successor, gen) != successor[6][gen] for gen in range(GENERATIONS)):
                    labels, cursor = [label], state
                    while parents[cursor] is not None:
                        cursor, previous = parents[cursor]
                        labels.append(previous)
                    first_hit = labels[::-1]
        frontier = next_frontier
    return first_hit, len(parents)


def main():
    """用法：c161_r3_mono.py <穷举深度>"""
    depth, emitted = int(sys.argv[1]), 0
    for name, kind, extra in (('bing_T6', 'bing', {}), ('jia_R4_T/discard=fe', 'jia', {'discard': 'fe'}),
                              ('jia_R4_T/discard=direct', 'jia', {'discard': 'direct'}),
                              ('yi_D_T', 'yiD', {}), ('yi_G_T', 'yiG', {})):
        for where_max in ('buffer', 'everywhere'):
            for tomb in ('wins', 'loses', 'order'):
                arm = dict(extra, kind=kind, where_max=where_max, tomb=tomb)
                hit, states = search(arm, depth)
                print('part=C arm=%s where_max=%s tomb=%s K=%d depth=%d states=%d %s' % (
                    name, where_max, tomb, K, depth, states,
                    'stale=%d:%s' % (len(hit), ','.join(hit)) if hit else 'none'), flush=True)
                emitted += 1
    print('emitted=%d' % emitted)


if __name__ == '__main__':
    main()
