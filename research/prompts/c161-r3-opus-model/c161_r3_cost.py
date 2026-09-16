#!/usr/bin/env python3
"""C161（缓冲两级并存而衔接没定） 第三轮攻方腿 · E 部分：U3 的三笔账（只数写出的节点，不含读、不含缓存）。

E1 乙-G-T 的 8 字节 seq 与落盘计数器：条目宽、叶容量、缓冲容量（纯算术）。
E2 T1 在（级联）塌缩下要多写多少节点：根只剩一个孩子时，把根缓冲推进孩子，孩子放不下就把消息最多的那组推下去、级联。
E3 丙-T6 在有删除的负载下每次发布排空到叶的写出量（反向索引覆盖写：每次一放一删），与乙排空到根缓冲比。
计数函数 nodes_written_drain_to_leaf / nodes_written_drain_to_root_buffer 原样抄自第一轮模型（c161-r1-opus-model/c161_model.py）。"""
import random
import sys

NODE_BYTES = 16384                                  # D8 已定项 2


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


# ---------------------------------------------------------------- E1：8 字节 seq
def e1_rows(emit):
    code2_header = 115 + 2 * 22                     # D8 已定项 11：含预留位的码 2 头 = 115 + 2 × key 宽；记账 key 22
    for label, seq_bytes in (('seq 4 字节（今天）', 4), ('seq 8 字节（T3）', 8)):
        entry = 22 + 8 + seq_bytes                  # key 22 + 完整值 8 + seq（D8 已定项 7：34 字节/条）
        emit('part=E1 case=%s accounting_entry_bytes=%d leaf_entries=%d buffer_messages_eps065=%d' % (
            label, entry, (NODE_BYTES - code2_header) // entry, buffer_messages_for(0.65, entry)))
    for message in (34, 109):                       # D11 已定项 7 的消息宽区间两端
        emit('part=E1 case=消息宽 %d→%d buffer_messages_eps065=%d→%d' % (
            message, message + 4, buffer_messages_for(0.65, message), buffer_messages_for(0.65, message + 4)))
    for rate in (750_000, 3_000_000):
        emit('part=E1 case=计数器回绕 rate_per_s=%d seq4_serial_window_s=%.0f seq8_serial_window_years=%.3g' % (
            rate, 2 ** 31 / rate, 2 ** 63 / rate / 86400 / 365.25))


# ---------------------------------------------------------------- E2：T1 塌缩的写出量
def spread(generator, count, fanout):
    groups = [0] * fanout
    for _ in range(count):
        groups[generator.randrange(fanout)] += 1
    return groups


def push_into(generator, fanout, capacity, levels_below, below_fill, incoming_groups, written):
    """一个内部节点收到按孩子分好组的消息；它自己先按 below_fill 预装；超了就把最多的那组推给那个孩子（级联）。"""
    written[0] += 1
    buffer = spread(generator, int(below_fill * capacity), fanout)
    buffer = [own + extra for own, extra in zip(buffer, incoming_groups)]
    while sum(buffer) > capacity:
        child = max(range(fanout), key=lambda index: (buffer[index], -index))
        count, buffer[child] = buffer[child], 0
        if levels_below == 0:
            written[0] += 1                          # 孩子是叶：写这片叶
        else:
            push_into(generator, fanout, capacity, levels_below - 1, below_fill, spread(generator, count, fanout), written)


def t1_collapse(fanout, capacity, levels_below_child, root_fill, below_fill, chain, trials, seed):
    """根只剩一个孩子。chain = 1：普通塌缩；chain = 2：根下一条单孩子链两层一起塌（先塌根、再塌新根）。"""
    generator = random.Random(seed)
    totals = []
    for _ in range(trials):
        written = [0]
        root_messages = int(root_fill * capacity)
        if chain == 1:
            push_into(generator, fanout, capacity, levels_below_child, below_fill,
                      spread(generator, root_messages, fanout), written)
        else:
            unary_messages = int(below_fill * capacity) + root_messages      # 中间那个单孩子节点的消息都归同一个孩子
            written[0] += 1
            push_into(generator, fanout, capacity, levels_below_child, below_fill,
                      spread(generator, unary_messages, fanout), written)
        totals.append(written[0])
    return sum(totals) / trials, max(totals)


def e2_rows(emit):
    for label, pivot, message in (('今天的宽 pivot 117 / 消息 34', 117, 34), ('E56 的宽 pivot 48 / 消息 16', 48, 16)):
        fanout, capacity = fanout_for(0.65, pivot), buffer_messages_for(0.65, message)
        for chain in (1, 2):
            for root_fill in (0.5, 1.0):
                for below_fill in (0.5, 0.9):
                    mean, worst = t1_collapse(fanout, capacity, 1, root_fill, below_fill, chain, 400, 20260915)
                    emit('part=E2 widths=%s F=%d B=%d chain=%d root_fill=%.1f below_fill=%.1f nodes_written_mean=%.2f max=%d' % (
                        label.replace(' ', ''), fanout, capacity, chain, root_fill, below_fill, mean, worst))


# ---------------------------------------------------------------- E3：有删除的负载
def accounting_touch(groups, generations_kept, capacity, seed, trials=200):
    """记账行按 (统计量, 维度, 代) 排：每组 K + 1 行连续。一次发布给每组写新一代、点删最老一代。
    叶按 [容量/2, 容量] 随机切。返回（只写不删碰到的叶，写 + 删碰到的叶，叶数）的均值。"""
    generator = random.Random(seed)
    rows = groups * (generations_kept + 1)
    only_put = with_delete = leaves_total = 0
    for _ in range(trials):
        boundaries, position = [], 0
        while position < rows:
            position += generator.randint(capacity // 2, capacity)
            boundaries.append(position)
        leaf_of = lambda row: next(index for index, end in enumerate(boundaries) if row < end)
        put_leaves = {leaf_of(group * (generations_kept + 1) + generations_kept) for group in range(groups)}
        delete_leaves = {leaf_of(group * (generations_kept + 1)) for group in range(groups)}
        only_put += len(put_leaves)
        with_delete += len(put_leaves | delete_leaves)
        leaves_total += len(boundaries)
    return only_put / trials, with_delete / trials, leaves_total / trials


def e3_rows(emit):
    fanout, capacity = fanout_for(0.65, 117), buffer_messages_for(0.65, 34)
    for overwrites in (1, 4, 16, 64):
        for label, entries in (('只插入', overwrites), ('覆盖写一放一删', 2 * overwrites)):
            leaf_cost, height = nodes_written_drain_to_leaf(32768, fanout_for(0.0, 117), entries, 400, 11)
            root_cost, root_height = nodes_written_drain_to_root_buffer(32768, fanout, capacity, entries, 400, 11)
            emit('part=E3 tree=反向索引 leaves=32768 per_publish=%d workload=%s entries=%d drain_to_leaf=%.3f(h%d) '
                 'drain_to_root_buffer=%.3f(h%d) ratio=%.3f' % (overwrites, label, entries, leaf_cost, height,
                                                                root_cost, root_height, leaf_cost / root_cost))
    leaf_entries = (NODE_BYTES - (115 + 2 * 22)) // 34
    for groups in (15, 18, 19, 64, 1000):
        put, both, leaves = accounting_touch(groups, 25, leaf_entries, 5)
        emit('part=E3 tree=记账 groups=%d K=25 rows=%d leaf_entries=%d leaves_mean=%.2f touched_put_only=%.2f '
             'touched_put_and_discard=%.2f' % (groups, groups * 26, leaf_entries, leaves, put, both))


# ---------------------------------------------------------------- E4：脏页预算下一个 checkpoint 装得下多少前端条目
def expected_drain_to_leaf(leaf_count, fanout, entries):
    """key 均匀时排空到叶碰到的节点数的期望：每层 n 个节点里被碰到的期望 n (1 − (1 − 1/n)^N)。"""
    total, nodes = 0.0, leaf_count
    while True:
        total += nodes * (1 - (1 - 1 / nodes) ** entries)
        if nodes == 1:
            return total
        nodes = -(-nodes // fanout)


def e4_rows(emit):
    budget_nodes = (256 << 20) // NODE_BYTES         # 默认环下有效 T_dirty = min(2 GiB, 768 MiB ÷ 3) = 256 MiB（D23 已定项 19 ③）
    fanout0, fanout, capacity = fanout_for(0.0, 117), fanout_for(0.65, 117), buffer_messages_for(0.65, 34)
    low, high = 1, 1 << 24
    while low < high:                                # 丙 / 甲：排空到叶的期望节点数 ≤ 预算的最大条目数
        middle = (low + high + 1) // 2
        low, high = (middle, high) if expected_drain_to_leaf(32768, fanout0, middle) <= budget_nodes else (low, middle - 1)
    emit('part=E4 arm=丙/甲（排空到叶） leaves=32768 budget_nodes=%d max_entries_per_checkpoint=%d records=%d' % (
        budget_nodes, low, -(-budget_nodes // 67)))
    for entries in (16384, 65536, 131072, 262144):
        written, _ = nodes_written_drain_to_root_buffer(32768, fanout, capacity, entries, 3, 13)
        emit('part=E4 arm=乙（排空到根缓冲） leaves=32768 entries_per_checkpoint=%d nodes_written=%.1f within_budget=%s' % (
            entries, written, written <= budget_nodes))
    emit('part=E4 case=按字节算 T_dirty 装得下的前端条目 entries=%d drain_to_leaf_nodes=%.0f' % (
        (256 << 20) // 34, expected_drain_to_leaf(32768, fanout0, (256 << 20) // 34)))


def main():
    rows = []
    for part in (e1_rows, e2_rows, e3_rows, e4_rows):
        part(lambda line: (rows.append(line), print(line, flush=True)))
    print('emitted=%d' % len(rows))


if __name__ == '__main__':
    main()
