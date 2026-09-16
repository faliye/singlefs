#!/usr/bin/env python3
"""C161（缓冲两级并存而衔接没定） 第三轮攻方腿 · B 部分：实例切换与管理员回退时，缓冲与计数器跟着回到哪里。

树固定成「根缓冲 → 一片叶」（丙的前端树 ε = 0，根缓冲不用）；写两个 key。
操作：前端写 / 删、甲的不走前端的写、flush、下推、开始发布（冻结这个 checkpoint 的镜像，开放下一个窗口）、
发布完成（进根环）、实例切换（D23 已定项 14：所选根取在飞 checkpoint 所基于的根，重发在飞 checkpoint，
号 > W 的事务 = 开放窗口里的写按新写序重做）、崩溃、管理员回退（选根环里前一个根，第一个新根同一次发布）、
挂回被抛弃的根（C332 那一格：回退实例的根全读不出）。
乙-G 的两个读法开关：切换时计数器接着内存里的走（continue）还是从所选根重新载入（reload，「挂载内做一次恢复」）；
重做的写重新取号（retake，T3「写进前端时取号」）还是留原号（keep）。回退后计数器取回退根的（selected）还是根环最大（ring_max）。
判据：U1 读对；U2 崩溃恢复后读对；T3 内存计数器 ≥ 活树与前端里任何一条 seq、盘上计数器 ≥ 那个根里任何一条 seq；
G3 丙的前端树叶上的逻辑改动对得上排空的前端条目——三种写法：按 checkpoint 累计、认窗口中途 flush 的（G3）；
只认发布时排空的（G3_publish_only）；只认这一次尝试的、重发时清零（G3_attempt）。"""
import sys

from c161_r3_tree import (TIE, adjacency_preserving_map, all_paths, cascade, flush_entry, g3_verdicts, is_leaf,
                          make_arm, make_inner, make_leaf, push_node, read_key, visible)

WRITE_KEYS = (0, 1)
KEY_NAMES = 'abc'
EMPTY_TREE = make_inner(0, 3, (), (make_leaf(0, 3, ()),))
INITIAL_TRUTH = ((0, True), (0, True), (0, True))

# 状态：(活树, 前端, 计数器, 在飞, 根环, 真值, 开放窗口日志, 开放窗口里 flush 到叶的条目)
#   在飞：None 或 (冻结镜像, 冻结时计数器, 冻结时真值, 这个 checkpoint 到叶的条目（flush + 发布时排空，跨重发累计）,
#                  发布时排空的条目（不含窗口中途 flush 的，跨重发累计）, 这一次尝试到叶的条目（flush + 排空，重发时清零）)
#   根环：((树, 计数器, 真值, 被抛弃), ...) 按发布先后；所选根 = 最后一个没被抛弃的
#   开放窗口日志：((key, version, is_delete, seq, 走哪条路), ...)：实例切换时号 > W 的事务照它重做


def map_tree(node, map_version, map_seq):
    if is_leaf(node):
        return ('L', node[1], node[2], tuple((key, map_version(key, version)) for key, version in node[3]))
    return ('I', node[1], node[2],
            tuple((m[0], map_version(m[0], m[1]), m[2], map_seq(m[3])) for m in node[3]),
            tuple(map_tree(child, map_version, map_seq) for child in node[4]))


def map_state(state, map_version, map_seq):
    """按状态布局把每个版本号、每个 seq 过一遍映射；收集时传记录函数，重编号时传查表函数。"""
    live, frontend, counter, inflight, ring, truth, log, flushed_open = state

    def entries(items):
        return tuple((e[0], map_version(e[0], e[1]), e[2], map_seq(e[3])) + tuple(e[4:]) for e in items)

    def truths(pairs):
        return tuple((map_version(key, pair[0]), pair[1]) for key, pair in enumerate(pairs))

    def applied(pairs):
        return tuple((key, None if value is None else map_version(key, value)) for key, value in pairs)

    if inflight is not None:
        frozen, frozen_counter, frozen_truth, drained_checkpoint, drained_publish, drained_attempt = inflight
        inflight = (map_tree(frozen, map_version, map_seq), map_seq(frozen_counter), truths(frozen_truth),
                    applied(drained_checkpoint), applied(drained_publish), applied(drained_attempt))
    ring = tuple((map_tree(tree, map_version, map_seq), map_seq(root_counter), truths(root_truth), abandoned)
                 for tree, root_counter, root_truth, abandoned in ring)
    return (map_tree(live, map_version, map_seq), entries(frontend), map_seq(counter), inflight, ring,
            truths(truth), entries(log), applied(flushed_open))


def relabel(state):
    versions, seqs = {}, set()

    def record_version(key, version):
        versions.setdefault(key, set()).add(version)
        return version

    def record_seq(seq):
        seqs.add(seq)
        return seq

    map_state(state, record_version, record_seq)
    version_maps = {key: {version: rank for rank, version in enumerate(sorted(values | {0}))}
                    for key, values in versions.items()}
    seq_map = adjacency_preserving_map(seqs)
    return map_state(state, lambda key, version: version_maps[key][version], lambda seq: seq_map[seq])


def next_version(state, key):
    found = [0]
    map_state(state, lambda item_key, version: found.append(version) if item_key == key else None, lambda seq: seq)
    return 1 + max(value for value in found if value is not None)


def selected_root(state):
    return [root for root in state[4] if not root[3]][-1]


def max_seq_in(tree):
    return max([m[3] for _, node in all_paths(tree) if not is_leaf(node) for m in node[3]] + [0])


def set_truth(truth, key, version, is_delete):
    truth = list(truth)
    truth[key] = (version, is_delete)
    return tuple(truth)


# ---------------------------------------------------------------- 操作
def op_write(arm, state, key, is_delete, via):
    live, frontend, counter, inflight, ring, truth, log, flushed_open = state
    version = next_version(state, key)
    seq = 0
    if arm['kind'] == 'yiG' and via == 'fe':
        counter += 1
        seq = counter
    entry = (key, version, is_delete, seq)
    if via == 'fe':
        frontend = tuple(e for e in frontend if e[0] != key) + (entry,)
    else:                                               # 甲：不走前端的写进根缓冲，逐出前端里的同 key
        frontend = tuple(e for e in frontend if e[0] != key)
        live = make_inner(live[1], live[2], live[3] + (entry,), live[4])
    return (live, frontend, counter, inflight, ring, set_truth(truth, key, version, is_delete), log + (entry + (via,),),
            flushed_open)


def op_flush(arm, state):
    live, frontend = state[0], state[1]
    if not frontend:
        return None
    live = flush_entry(arm, live, frontend[0], True)
    if live is None or live is TIE:
        return live
    flushed_open = state[7] + (((frontend[0][0], visible(frontend[0])),) if arm['kind'] == 'bing' else ())
    return (live, frontend[1:]) + state[2:7] + (flushed_open,)


def op_push(arm, state):
    pushed = push_node(arm, state[0], 0, True)
    if pushed is None or pushed is TIE:
        return pushed
    return (pushed,) + state[1:]


def drain_front_end(arm, live, frontend):
    for entry in frontend:
        live = flush_entry(arm, live, entry, False)
        if live is TIE:
            return TIE
    if arm['kind'] in ('yiD', 'yiG'):
        live = cascade(arm, live)
    return live


def op_begin_publish(arm, state):
    live, frontend, counter, inflight, ring, truth, log, flushed_open = state
    if inflight is not None:
        return None
    live = drain_front_end(arm, live, frontend)
    if live is TIE:
        return TIE
    drained = tuple((e[0], visible(e)) for e in frontend) if arm['kind'] == 'bing' else ()
    return (live, (), counter, (live, counter, truth, flushed_open + drained, drained, flushed_open + drained), ring, truth,
            (), ())


def op_finish_publish(arm, state):
    live, frontend, counter, inflight, ring, truth, log, flushed_open = state
    if inflight is None:
        return None, ()
    frozen, frozen_counter, frozen_truth, drained_checkpoint, drained_publish, drained_attempt = inflight
    flags = ()
    if arm['kind'] == 'bing':
        base = selected_root(state)[0]
        flags += ('G3',) if g3_verdicts(base, frozen, drained_checkpoint)[0] else ()
        flags += ('G3_publish_only',) if g3_verdicts(base, frozen, drained_publish)[0] else ()
        flags += ('G3_attempt',) if g3_verdicts(base, frozen, drained_attempt)[0] else ()
    return (live, frontend, counter, None, ring + ((frozen, frozen_counter, frozen_truth, False),), truth, log,
            flushed_open), flags


def op_switch(arm, state):
    """实例切换：所选根取在飞 checkpoint 所基于的根，重发在飞 checkpoint（冻结镜像原样），开放窗口里的写按新写序重做。"""
    live, frontend, counter, inflight, ring, truth, log, _ = state
    if inflight is None:
        return None
    frozen, frozen_counter, frozen_truth, drained_checkpoint, drained_publish, _ = inflight
    if arm['switch_counter'] == 'reload':               # 「挂载内做一次恢复」：计数器从所选根重新载入
        counter = selected_root(state)[1]
    live, frontend, redone = frozen, (), []
    for key, version, is_delete, seq, via in log:
        if arm['kind'] == 'yiG' and via == 'fe' and arm['redo_seq'] == 'retake':
            counter += 1
            seq = counter
        entry = (key, version, is_delete, seq)
        if via == 'fe':
            frontend = tuple(e for e in frontend if e[0] != key) + (entry,)
        else:
            frontend = tuple(e for e in frontend if e[0] != key)
            live = make_inner(live[1], live[2], live[3] + (entry,), live[4])
        redone.append(entry + (via,))
    inflight = (frozen, frozen_counter, frozen_truth, drained_checkpoint, drained_publish, ())
    return (live, frontend, counter, inflight, ring, truth, tuple(redone), ())


def op_crash(arm, state):
    tree, root_counter, root_truth, _ = selected_root(state)
    return (tree, (), root_counter, None, state[4], root_truth, (), ())


def op_rollback(arm, state):
    """管理员回退（挂载时带外）：选所选根之前的那个没被抛弃的根 R_old，之后的根判被抛弃；第一个新根与回退同一次发布（这里空载荷）。"""
    live_roots = [index for index, root in enumerate(state[4]) if not root[3]]
    if len(live_roots) < 2:
        return None
    old_index = live_roots[-2]
    ring = tuple((root[0], root[1], root[2], root[3] or index > old_index) for index, root in enumerate(state[4]))
    tree, root_counter, root_truth, _ = ring[old_index]
    if arm['rollback_counter'] == 'ring_max':
        root_counter = max(root[1] for root in ring)
    ring += ((tree, root_counter, root_truth, False),)
    return (tree, (), root_counter, None, ring, root_truth, (), ())


def op_remount_abandoned(arm, state):
    """C332 那一格：回退实例的根全读不出，挂载挂上被抛弃时间线里最新的根。"""
    abandoned = [index for index, root in enumerate(state[4]) if root[3]]
    if not abandoned:
        return None
    pick = abandoned[-1]
    ring = tuple((root[0], root[1], root[2], index != pick and (root[3] or index > pick))
                 for index, root in enumerate(state[4]))
    tree, root_counter, root_truth, _ = ring[pick]
    return (tree, (), root_counter, None, ring, root_truth, (), ())


# ---------------------------------------------------------------- 判据、枚举、主程序
def violations(arm, state):
    live, frontend, counter, inflight, ring, truth, _, _ = state
    found = []
    for key in WRITE_KEYS:
        values = read_key(arm, (live, frontend), key)
        expected = None if truth[key][1] else truth[key][0]
        if len(values) > 1:
            found.append('undefined')
        elif expected not in values:
            found.append('stale')
    if arm['kind'] == 'yiG':
        in_memory = [e[3] for e in frontend] + [max_seq_in(live)] + ([max_seq_in(inflight[0])] if inflight else [])
        if counter < max(in_memory):
            found.append('T3_memory')
        if any(root_counter < max_seq_in(tree) for tree, root_counter, _, abandoned in ring if not abandoned):
            found.append('T3_disk')
    return found


def successors(arm, state):
    for key in WRITE_KEYS:
        for is_delete in (False, True):
            verb = 'del' if is_delete else 'put'
            yield 'fe_%s_%s' % (verb, KEY_NAMES[key]), 'write', op_write(arm, state, key, is_delete, 'fe'), ()
            if arm['kind'] == 'jia':
                yield 'direct_%s_%s' % (verb, KEY_NAMES[key]), 'write', op_write(arm, state, key, is_delete, 'direct'), ()
    yield 'flush', 'flush', op_flush(arm, state), ()
    if arm['kind'] != 'bing':
        yield 'push', 'push', op_push(arm, state), ()
    yield 'begin_publish', 'publish', op_begin_publish(arm, state), ()
    finished, flags = op_finish_publish(arm, state)
    yield 'finish_publish', 'publish', finished, flags
    yield 'switch', 'switch', op_switch(arm, state), ()
    yield 'crash', 'crash', op_crash(arm, state), ()
    yield 'rollback', 'rollback', op_rollback(arm, state), ()
    yield 'remount_abandoned', 'remount', op_remount_abandoned(arm, state), ()


def trim_ring(state):
    """根环只留最近 3 个根（D22 已定项 2 的 R = 3）。"""
    return state[:4] + (state[4][-3:],) + state[5:]


def breadth_first_search(arm, depth_limit):
    start = relabel((EMPTY_TREE, (), 0, None, ((EMPTY_TREE, 0, INITIAL_TRUTH, False),), INITIAL_TRUTH, (), ()))
    parents, frontier, first_hits = {start: None}, [start], {}

    def path_to(state):
        labels = []
        while parents[state] is not None:
            state, label = parents[state]
            labels.append(label)
        return labels[::-1]

    for _ in range(depth_limit):
        next_frontier = []
        for state in frontier:
            for label, kind, successor, flags in successors(arm, state):
                if successor is None:
                    continue
                if successor is TIE:
                    first_hits.setdefault('undefined', path_to(state) + [label])
                    continue
                successor = relabel(trim_ring(successor))
                found = list(flags) + violations(arm, successor)
                if kind in ('crash', 'rollback', 'remount') and ('stale' in found or 'undefined' in found):
                    found.append('u2_recovered')
                for category in found:
                    first_hits.setdefault(category, path_to(state) + [label])
                if successor not in parents:
                    parents[successor] = (state, label)
                    next_frontier.append(successor)
        frontier = next_frontier
    return first_hits, len(parents)


def b_arm(kind, switch_counter='continue', redo_seq='retake', rollback_counter='selected'):
    return make_arm(kind, switch_counter=switch_counter, redo_seq=redo_seq, rollback_counter=rollback_counter)


PART_B_ARMS = [
    ('bing_T6', b_arm('bing')),
    ('jia_R4_T', b_arm('jia')),
    ('yi_D_T', b_arm('yiD')),
    ('yi_G_T/switch=continue/redo=retake/rollback=selected', b_arm('yiG')),
    ('yi_G_T/switch=continue/redo=keep/rollback=selected', b_arm('yiG', redo_seq='keep')),
    ('yi_G_T/switch=reload/redo=retake/rollback=selected', b_arm('yiG', switch_counter='reload')),
    ('yi_G_T/switch=reload/redo=keep/rollback=selected', b_arm('yiG', switch_counter='reload', redo_seq='keep')),
    ('yi_G_T/switch=continue/redo=retake/rollback=ring_max', b_arm('yiG', rollback_counter='ring_max')),
]

HIT_ORDER = ('stale', 'undefined', 'u2_recovered', 'T3_memory', 'T3_disk', 'G3', 'G3_publish_only', 'G3_attempt')


def main():
    """用法：c161_r3_switch.py <穷举深度> <甲的穷举深度>（甲多一种写，状态数大一个量级）"""
    emitted = 0
    chosen = [int(index) for index in sys.argv[3].split(',')] if len(sys.argv) > 3 else range(len(PART_B_ARMS))
    for name, arm in [PART_B_ARMS[index] for index in chosen]:
        depth = int(sys.argv[2]) if arm['kind'] == 'jia' else int(sys.argv[1])
        hits, states = breadth_first_search(arm, depth)
        parts = ['%s=%d:%s' % (category, len(hits[category]), ','.join(hits[category]))
                 for category in HIT_ORDER if category in hits]
        print('part=B arm=%s depth=%d states=%d %s' % (name, depth, states, ' '.join(parts) if parts else 'none'), flush=True)
        emitted += 1
    print('emitted=%d' % emitted)


if __name__ == '__main__':
    main()
