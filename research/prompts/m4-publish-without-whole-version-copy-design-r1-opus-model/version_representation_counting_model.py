#!/usr/bin/env python3
"""三条候选表示在同一串发布上的计数模型（纯算术、只用标准库、不读仓里任何文件）。

候选（照背景材料「问题」第 2 条的定义）：
  整版  今天的样子：每一版是一份整的数组，发布照抄全部条目再改。
  甲    持久共享结构：完全 16 叉、深 3 层的持久树（16^3 = 4096 个键），改一个键只拷根到叶那一条路，没改的节点与上一版共用。
  乙    版本只带这次的差量（键、旧值、新值），全版由写者持有的一份可变索引（dict）给。
  丙    版本带差量加指向上一版的引用；起点那一版的差量就是全表；按键查沿链往回找，读全版沿链合出。

同一串发布：起点版本有 N0 个键；之后 PUBLISHES 次发布，第 t 次（t 从 1 起）只改键 (t - 1) % HOT_KEYS。
键 HOT_KEYS 以上的从起点之后一次都没改过（冷键）。

每一格打一行 `name=<格名> <量>=<值>`，并对每个值断言绝对值（断言的式子写在旁边）；断言不过退 1。
数的是条目与节点的个数，不是挂钟。量的是这份模型，不是 crates/ 里的实现。
"""
import sys

N0 = 4096
FANOUT = 16
DEPTH = 3
PUBLISHES = 4096
HOT_KEYS = 64
RELOCATED_KEYS = tuple(range(100, 116))   # 整理一批搬的 16 个冷键（批大小同产品默认 16）
VERSIONS_IN_ONE_CHANGE = 3                 # 一次用户改动交回的几版（抬 F 的空发布、走空的一步、改动自己）

failures = []


def check(name, quantity, value, expected):
    print(f"name={name} {quantity}={value} expected={expected}")
    if value != expected:
        failures.append(f"{name} {quantity}: 量到 {value}，断言 {expected}")


def key_written_by_publish(t):
    return (t - 1) % HOT_KEYS


# ── 甲：持久树 ──
class PersistentNode:
    __slots__ = ("slots",)

    def __init__(self, slots):
        self.slots = slots


def path_digits(key):
    return [(key // FANOUT ** (DEPTH - 1 - level)) % FANOUT for level in range(DEPTH)]


def persistent_base():
    leaves = [PersistentNode([("base", key) for key in range(start, start + FANOUT)])
              for start in range(0, N0, FANOUT)]
    middle = [PersistentNode(leaves[start:start + FANOUT]) for start in range(0, len(leaves), FANOUT)]
    return PersistentNode(middle)


def persistent_update(root, key, value, copied):
    digits = path_digits(key)

    def copy_down(node, level):
        fresh = PersistentNode(list(node.slots))
        copied.append(fresh)
        digit = digits[level]
        if level == DEPTH - 1:
            fresh.slots[digit] = value
        else:
            fresh.slots[digit] = copy_down(node.slots[digit], level + 1)
        return fresh

    return copy_down(root, 0)


def persistent_reachable_nodes(roots):
    seen = set()
    stack = list(roots)
    while stack:
        node = stack.pop()
        if id(node) in seen:
            continue
        seen.add(id(node))
        if isinstance(node.slots[0], PersistentNode):
            stack.extend(node.slots)
    return len(seen)


def persistent_distinct_path_nodes(keys):
    """改这几个键要拷的节点：各键根到叶那条路的并。"""
    nodes = set()
    for key in keys:
        digits = path_digits(key)
        for level in range(DEPTH):
            nodes.add((level, tuple(digits[:level])))
    return len(nodes)


def persistent_equal_node_pairs(left, right):
    """逐节点比，指着同一个节点就不往下走；交回比过的节点对数。"""
    if left is right:
        return 1
    pairs = 1
    if isinstance(left.slots[0], PersistentNode):
        for left_child, right_child in zip(left.slots, right.slots):
            pairs += persistent_equal_node_pairs(left_child, right_child)
    return pairs


# ── 丙：差量链 ──
class ChainVersion:
    __slots__ = ("delta", "previous")

    def __init__(self, delta, previous):
        self.delta = delta
        self.previous = previous


def chain_lookup_hops(version, key):
    hops = 0
    while key not in version.delta:
        version = version.previous
        hops += 1
    return hops


def chain_materialize(version):
    """沿链合出全版：交回（看过的版数，看过的差量条目数）。"""
    seen = set()
    versions_visited = 0
    entries_examined = 0
    while version is not None:
        versions_visited += 1
        for key in version.delta:
            entries_examined += 1
            seen.add(key)
        version = version.previous
    assert len(seen) == N0
    return versions_visited, entries_examined


def main():
    # ── 同一串发布：P 次，每次改一个热键 ──
    whole_current = [("base", key) for key in range(N0)]
    whole_entries_copied = 0
    whole_entries_retained_all = len(whole_current)
    jia_roots = [persistent_base()]
    jia_nodes_copied = 0
    yi_index = {key: ("base", key) for key in range(N0)}
    yi_deltas = []
    yi_operations = 0
    bing_latest = ChainVersion({key: ("base", key) for key in range(N0)}, None)
    bing_operations = 0
    for t in range(1, PUBLISHES + 1):
        key = key_written_by_publish(t)
        value = ("publish", t)
        whole_current = list(whole_current)
        whole_entries_copied += len(whole_current)
        whole_current[key] = value
        whole_entries_retained_all += len(whole_current)
        copied = []
        jia_roots.append(persistent_update(jia_roots[-1], key, value, copied))
        jia_nodes_copied += len(copied)
        yi_deltas.append((key, yi_index[key], value))
        yi_index[key] = value
        yi_operations += 2
        bing_latest = ChainVersion({key: value}, bing_latest)
        bing_operations += 1
    check("publish_whole_version", "entries_copied_per_publish", whole_entries_copied // PUBLISHES, N0)
    check("publish_whole_version", "entries_copied_total", whole_entries_copied, N0 * PUBLISHES)          # 16777216
    check("publish_jia", "nodes_copied_per_publish", jia_nodes_copied // PUBLISHES, DEPTH)                 # 3
    check("publish_jia", "slots_copied_total", jia_nodes_copied * FANOUT, DEPTH * FANOUT * PUBLISHES)     # 196608
    check("publish_yi", "operations_total", yi_operations, 2 * PUBLISHES)                                   # 8192
    check("publish_bing", "operations_total", bing_operations, PUBLISHES)                                   # 4096

    # ── 留着每一版（模型对拍与验收拿到的版本列表那一类持有者）──
    base_nodes = 1 + FANOUT + N0 // FANOUT                                                                  # 273
    check("retain_every_version_whole", "entries", whole_entries_retained_all, (PUBLISHES + 1) * N0)       # 16781312
    check("retain_every_version_jia", "nodes", persistent_reachable_nodes(jia_roots), base_nodes + DEPTH * PUBLISHES)  # 12561
    check("retain_every_version_yi", "entries_index_plus_deltas", len(yi_index) + len(yi_deltas), N0 + PUBLISHES)    # 8192
    check("retain_every_version_bing", "entries", sum(len(v.delta) for v in iter_chain(bing_latest)), N0 + PUBLISHES)  # 8192
    # ── 只留最新一版（别的持有者都放手了）：丙的最新一版仍拽着整条链 ──
    check("retain_latest_only_jia", "nodes", persistent_reachable_nodes([jia_roots[-1]]), base_nodes)     # 273
    check("retain_latest_only_bing", "entries_kept_alive_by_the_chain",
          sum(len(v.delta) for v in iter_chain(bing_latest)), N0 + PUBLISHES)                              # 8192，随发布次数涨

    # ── 丙：按键随机找、读全版 ──
    check("bing_lookup_cold_key", "hops", chain_lookup_hops(bing_latest, 1000), PUBLISHES)                 # 4096
    check("bing_lookup_hot_key_written_first", "hops", chain_lookup_hops(bing_latest, 0), HOT_KEYS - 1)    # 63
    check("bing_lookup_hot_key_written_last", "hops", chain_lookup_hops(bing_latest, HOT_KEYS - 1), 0)
    check("bing_lookup_every_key", "hops_total", sum(chain_lookup_hops(bing_latest, key) for key in range(N0)),
          HOT_KEYS * (HOT_KEYS - 1) // 2 + (N0 - HOT_KEYS) * PUBLISHES)                                     # 16517088
    check("bing_materialize_latest", "versions_visited", chain_materialize(bing_latest)[0], PUBLISHES + 1)
    check("bing_materialize_latest", "entries_examined", chain_materialize(bing_latest)[1], N0 + PUBLISHES)

    # ── 整理一批：一版里每一处指针换成映射位置（take_the_mapping_positions_over_the_location_hints 的走法）──
    check("relocation_walk_whole", "entries_read_plus_previous_clone", N0 + N0, 2 * N0)
    check("relocation_walk_jia_mutating_every_node", "nodes_copied", persistent_reachable_nodes([jia_roots[-1]]), base_nodes)
    check("relocation_walk_jia_read_then_targeted_copy", "entries_read", N0, N0)
    check("relocation_walk_jia_targeted", "nodes_copied", persistent_distinct_path_nodes(RELOCATED_KEYS), 4)
    check("relocation_walk_yi", "entries_read", len(yi_index), N0)
    check("relocation_walk_bing", "entries_examined", chain_materialize(bing_latest)[1], N0 + PUBLISHES)
    with_reverse_index = len(RELOCATED_KEYS)
    check("relocation_with_a_key_to_pointer_index", "entries_read", with_reverse_index, 16)

    # ── 一次用户改动交回三版，验收在改动交回之后逐版读全版的分配记录 ──
    for t in range(PUBLISHES + 1, PUBLISHES + 1 + VERSIONS_IN_ONE_CHANGE):
        key = key_written_by_publish(t)
        value = ("publish", t)
        jia_roots.append(persistent_update(jia_roots[-1], key, value, []))
        yi_deltas.append((key, yi_index[key], value))
        yi_index[key] = value
        bing_latest = ChainVersion({key: value}, bing_latest)
    check("change_of_three_versions_whole", "entries_read", VERSIONS_IN_ONE_CHANGE * N0, 12288)
    check("change_of_three_versions_jia", "node_pairs_to_diff_consecutive",
          persistent_equal_node_pairs(jia_roots[-2], jia_roots[-1]), 33)
    yi_reconstruct = sum(N0 + undone for undone in range(VERSIONS_IN_ONE_CHANGE))
    check("change_of_three_versions_yi", "entries_to_reconstruct_with_undo_records", yi_reconstruct, 12291)
    bing_reads = 0
    walker = bing_latest
    for _ in range(VERSIONS_IN_ONE_CHANGE):
        bing_reads += chain_materialize(walker)[1]
        walker = walker.previous
    check("change_of_three_versions_bing", "entries_examined", bing_reads, 24582)

    # ── 发布失败换回 ──
    check("failure_swap_back_whole", "entries_cloned", N0, 4096)
    check("failure_swap_back_jia", "entries_cloned", 0, 0)
    check("failure_swap_back_yi_by_clone", "entries_cloned", len(yi_index), 4096)
    check("failure_swap_back_yi_by_undo_record", "entries_undone", 1, 1)
    check("failure_swap_back_bing", "entries_cloned", 0, 0)

    # ── 乙：被拒的一次改坏了写者的索引而没记差量，「现行那一版不动」那一判仍判相等 ──
    yi_version_before = tuple(yi_deltas[-1])
    yi_full_before = dict(yi_index)
    yi_index[5] = ("written_by_a_refused_publish", 0)
    check("yi_refused_publish_bug", "version_values_equal", int(tuple(yi_deltas[-1]) == yi_version_before), 1)
    check("yi_refused_publish_bug", "full_contents_equal", int(yi_index == yi_full_before), 0)

    # ── 甲：两份独立建的相等版本（从盘上重建的那一版与发布出来的那一版）没有共用节点，比满全部节点 ──
    check("jia_equal_without_shared_nodes", "node_pairs", persistent_equal_node_pairs(persistent_base(), persistent_base()), base_nodes)

    if failures:
        for failure in failures:
            print(f"  ✗ {failure}")
        return 1
    print("  ✓ 全部断言对上")
    return 0


def iter_chain(version):
    while version is not None:
        yield version
        version = version.previous


if __name__ == "__main__":
    sys.exit(main())
