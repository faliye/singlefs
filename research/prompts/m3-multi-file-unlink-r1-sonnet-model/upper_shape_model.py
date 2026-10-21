#!/usr/bin/env python3
"""extent 树上段形状的独立小模型（正推腿，m3-multi-file-unlink-r1）。

只用标准库，不读 crates/ 的任何代码：几何常量（U = 一片叶罩几个 inode 号，F = 内部扇出）是参数。
问的是：写与删共用一条上段形状规则时，
  规则 A：形状只由「有条目的 inode 集合 S」定（节点 = S 里每个号的叶到根那一串的并；根层级 = 罩得住 max(S) 的最低一层；S 空时一个零条目根兼叶）；
  规则 B：写路径今天的读法（根层级取 max(罩得住 max(S) 的最低一层, 上一版根层级)，S 空时才落回零条目根），
各自的「照抄 / 重写」按路径定，对不对（与按内容定的判法逐步相同）、写多少节点、有没有「同一个 S 两种形状」。
节点内容 = 叶：(inode, 该 inode 条目的令牌) 的升序元组；内部节点：(孩子位置, 孩子的出生号) 的升序元组。
照抄的节点出生号不变；重写的节点发新出生号。内容相同才算「真的没变」。

用法：python3 upper_shape_model.py [--seed N] [--steps N] [--selftest]
输出每行 name=... key=value，--selftest 逐个变异必须判红。
"""
import itertools
import random
import sys
from collections import OrderedDict


class Geometry:
    def __init__(self, leaf_inodes, fanout):
        self.U = leaf_inodes
        self.F = fanout

    def span(self, level):
        return self.U * self.F ** level

    def level_for(self, largest):
        level = 0
        while self.span(level) <= largest:
            level += 1
        return level

    def path(self, inode, root_level):
        """叶到根那一串位置，叶在前。"""
        assert self.span(root_level) > inode
        position = (0, inode // self.U)
        out = [position]
        while out[-1][0] < root_level:
            level, index = out[-1]
            out.append((level + 1, index // self.F))
        return out

    def parent(self, position):
        level, index = position
        return (level + 1, index // self.F)


class Version:
    def __init__(self, entries, positions, root_level, contents, ids):
        self.entries = dict(entries)        # inode -> 令牌（S 就是它的键）
        self.positions = frozenset(positions)
        self.root_level = root_level
        self.contents = dict(contents)      # 位置 -> 内容元组
        self.ids = dict(ids)                # 位置 -> 出生号


def canonical_positions(g, S, rule, previous_level):
    if not S:
        return frozenset({(0, 0)}), 0
    level = g.level_for(max(S))
    if rule == "B":
        level = max(level, previous_level)
    out = set()
    for inode in S:
        out.update(g.path(inode, level))
    return frozenset(out), level


def contents_from_scratch(g, entries, positions, child_ids):
    """按 entries 与孩子的出生号算每个位置的内容。child_ids: 位置 -> 出生号（叶用不到）。"""
    contents = {}
    for position in sorted(positions):
        level, index = position
        if level == 0:
            contents[position] = tuple(sorted(
                (inode, token) for inode, token in entries.items() if inode // g.U == index))
        else:
            contents[position] = tuple(sorted(
                (child, child_ids[child]) for child in positions
                if child[0] == level - 1 and g.parent(child) == position))
    return contents


class Planner:
    """mutant 是弄坏哪一处（None = 正确）。"""

    def __init__(self, g, rule, mutant=None):
        self.g = g
        self.rule = rule
        self.mutant = mutant
        self.next_id = 1

    def fresh(self):
        value = self.next_id
        self.next_id += 1
        return value

    def initial(self):
        entries = {}
        positions = frozenset({(0, 0)})
        ids = {(0, 0): self.fresh()}
        contents = contents_from_scratch(self.g, entries, positions, ids)
        return Version(entries, positions, 0, contents, ids)

    def step(self, previous, inode, kind):
        """kind: 'write'（新进 S 或覆盖）| 'delete'。交回 (新版本, 重写的位置集, 换下的上一版位置集, 违例清单)。"""
        g = self.g
        entries = dict(previous.entries)
        if kind == "write":
            entries[inode] = self.fresh()          # 这个号的条目令牌换新
        else:
            del entries[inode]
        positions, level = canonical_positions(g, set(entries), self.rule, previous.root_level)
        if self.mutant == "keep-empty-leaf" and kind == "delete":
            leaf = (0, inode // g.U)
            if leaf in previous.positions and not any(i // g.U == leaf[1] for i in entries):
                positions = positions | {leaf} | set(g.path(inode, previous.root_level))
                level = max(level, previous.root_level)
        if self.mutant == "shrink-keeps-old-root" and kind == "delete" and entries:
            positions, level = canonical_positions(g, set(entries), "B", previous.root_level)
        # 路径规则：变了的号那一串（按上一版根层级与新根层级两边算）里留在新形状中的位置重写，新形状里上一版没有的位置重写
        changed_path = set()
        for root_level in {previous.root_level, level}:
            if g.span(root_level) > inode:
                changed_path.update(g.path(inode, root_level))
        if self.mutant == "no-ancestor-rewrite":
            changed_path = {(0, inode // g.U)}
        rewritten = {p for p in positions if p in changed_path or p not in previous.positions}
        replaced = {p for p in previous.positions if p in rewritten or p not in positions}
        ids = {}
        for position in sorted(positions):               # 层级升序：孩子先于父
            ids[position] = self.fresh() if position in rewritten else previous.ids[position]
        contents = contents_from_scratch(g, entries, positions, ids)
        violations = []
        for position in positions:
            if position not in rewritten and contents[position] != previous.contents[position]:
                violations.append(("carried-node-content-differs", position))
        # 内容判法（按内容定的最小重写集）：自底向上，新内容与上一版同位置内容相同才算没变
        oracle_ids = {}
        oracle_rewritten = set()
        for position in sorted(positions):
            level_p, index_p = position
            if level_p == 0:
                content = contents[position]
            else:
                content = tuple(sorted(
                    (child, oracle_ids[child]) for child in positions
                    if child[0] == level_p - 1 and g.parent(child) == position))
            if position in previous.positions and previous.contents[position] == content:
                oracle_ids[position] = previous.ids[position]
            else:
                oracle_ids[position] = -position[0] * 1000003 - position[1] - 1   # 一个不会与旧号撞的占位
                oracle_rewritten.add(position)
        if oracle_rewritten != rewritten:
            violations.append(("path-rule-differs-from-content-rule",
                               tuple(sorted(rewritten ^ oracle_rewritten))[:3]))
        # 形状判：非根节点不许零条目；位置集合等于规则给的形状
        for position in positions:
            if position != (0, 0) or level != 0:
                if not contents[position]:
                    violations.append(("empty-node-below-or-above-the-leaf-root", position))
        if level == 0 and not entries and positions != frozenset({(0, 0)}):
            violations.append(("empty-set-not-a-single-zero-entry-root", tuple(sorted(positions))))
        if self.rule == "A" or self.mutant in ("keep-empty-leaf", "shrink-keeps-old-root"):
            expected, expected_level = canonical_positions(g, set(entries), "A", 0)
            if positions != expected or level != expected_level:
                violations.append(("shape-not-the-pure-function-of-the-entry-set",
                                   (level, expected_level)))
        return (Version(entries, positions, level, contents, ids), rewritten, replaced, violations)


def random_inode(rng, g, universe):
    # 混合分布：叶 0 里的小号、同层邻叶、更高一层、再高一层
    pick = rng.random()
    if pick < 0.35:
        return rng.randint(1, g.U - 1)
    if pick < 0.6:
        return rng.randint(g.U, g.span(1) - 1)
    if pick < 0.85:
        return rng.randint(g.span(1), g.span(2) - 1)
    return rng.randint(1, universe)


def run_history(g, rule, mutant, rng, steps, universe, collect):
    planner = Planner(g, rule, mutant)
    version = planner.initial()
    problems = []
    for _ in range(steps):
        present = sorted(version.entries)
        if present and rng.random() < 0.45:
            inode = rng.choice(present)
            kind = "delete"
        else:
            inode = random_inode(rng, g, universe)
            kind = "write"
        before = version
        version, rewritten, replaced, violations = planner.step(version, inode, kind)
        problems.extend(violations)
        collect(kind, before, version, rewritten, replaced, inode)
    return problems


def shape_key(version):
    return (tuple(sorted(version.entries)), version.positions, version.root_level)


def survey(g, rule, mutant, seeds, steps, universe):
    counters = OrderedDict(steps=0, deletes=0, writes=0, rewritten_on_delete=0, rewritten_on_write=0,
                           replaced_on_delete=0, max_rewritten_per_op=0, levels_dropped_by_delete=0,
                           levels_raised_by_write=0, violations=0)
    shapes_of_set = {}

    def collect(kind, before, after, rewritten, replaced, inode):
        counters["steps"] += 1
        key = tuple(sorted(after.entries))
        shapes_of_set.setdefault(key, set()).add((after.positions, after.root_level))
        counters["max_rewritten_per_op"] = max(counters["max_rewritten_per_op"], len(rewritten))
        if kind == "delete":
            counters["deletes"] += 1
            counters["rewritten_on_delete"] += len(rewritten)
            counters["replaced_on_delete"] += len(replaced)
            if after.root_level < before.root_level:
                counters["levels_dropped_by_delete"] += before.root_level - after.root_level
        else:
            counters["writes"] += 1
            counters["rewritten_on_write"] += len(rewritten)
            if after.root_level > before.root_level:
                counters["levels_raised_by_write"] += after.root_level - before.root_level

    all_problems = []
    for seed in seeds:
        rng = random.Random(seed)
        all_problems.extend(run_history(g, rule, mutant, rng, steps, universe, collect))
    counters["violations"] = len(all_problems)
    counters["entry_sets_seen"] = len(shapes_of_set)
    counters["entry_sets_with_two_shapes"] = sum(1 for shapes in shapes_of_set.values() if len(shapes) > 1)
    return counters, all_problems


def order_pair_demo(g, rule):
    """S = {1, 大号}：先删 1 再删大号，与先删大号再删 1，每一步的位置集合，与出生号是否相同。"""
    big = g.span(1) + 5
    out = {}
    for order in ((1, big), (big, 1)):
        planner = Planner(g, rule)
        version = planner.initial()
        version = planner.step(version, 1, "write")[0]
        version = planner.step(version, big, "write")[0]
        trail = []
        for inode in order:
            version, rewritten, replaced, violations = planner.step(version, inode, "delete")
            assert not violations, violations
            trail.append((tuple(sorted(version.positions)), version.root_level))
        out[order] = (trail, tuple(sorted(version.ids.items())))
    first, second = out[(1, big)], out[(big, 1)]
    return first[0][-1] == second[0][-1], first[1] != second[1], first[0], second[0]


# ---------- 下段批规则（K-R3-1）：与 inode 无关的部分 ----------
def lower_nodes(units, leaf_units, fanout):
    if units <= 1:
        return []
    level = 0
    while leaf_units * fanout ** level < units:
        level += 1
    out = []
    for lv in range(level + 1):
        span = leaf_units * fanout ** lv
        out.extend((lv, i) for i in range(-(-units // span)))
    return out


def lower_content(units_in_file, leaf_units, fanout, position, pointer_of_unit, child_ids):
    level, index = position
    span = leaf_units * fanout ** level
    first, last = index * span, min(units_in_file - 1, (index + 1) * span - 1)
    if level == 0:
        return tuple((u, pointer_of_unit[u]) for u in range(first, last + 1))
    child_span = leaf_units * fanout ** (level - 1)
    return tuple(((level - 1, c), child_ids[(level - 1, c)])
                 for c in range(first // child_span, last // child_span + 1))


def lower_batch_check(leaf_units, fanout, total_units, kept):
    """代码今天的规则：留下的单元全落在节点罩的一段之内才照抄（last_data_unit < kept）。内容判法逐点核。"""
    before_nodes = lower_nodes(total_units, leaf_units, fanout)
    pointer_of_unit = {u: 1000 + u for u in range(total_units)}
    before_ids = {}
    for position in before_nodes:  # 层级升序 = 孩子先
        pass
    ids = {}
    counter = [1]
    for position in sorted(before_nodes):
        ids[position] = counter[0]
        counter[0] += 1
    before_contents = {p: lower_content(total_units, leaf_units, fanout, p, pointer_of_unit, ids)
                       for p in sorted(before_nodes)}
    after_nodes = lower_nodes(kept, leaf_units, fanout)
    new_ids = {}
    rewritten = set()
    for position in sorted(after_nodes):
        level, index = position
        span = leaf_units * fanout ** level
        last_unit = (index + 1) * span - 1
        rule_carries = last_unit < kept and position in before_nodes
        if rule_carries:
            new_ids[position] = ids[position]
        else:
            new_ids[position] = counter[0]
            counter[0] += 1
            rewritten.add(position)
    # 内容判法
    oracle_rewritten = set()
    oracle_ids = {}
    for position in sorted(after_nodes):
        content = lower_content(kept, leaf_units, fanout, position, pointer_of_unit, oracle_ids)
        if position in before_contents and before_contents[position] == content:
            oracle_ids[position] = ids[position]
        else:
            oracle_ids[position] = -1 - len(oracle_ids)
            oracle_rewritten.add(position)
    return rewritten == oracle_rewritten, sorted(rewritten ^ oracle_rewritten)


def boundary_churn(g, rule, rounds):
    """S = {1}，反复写 / 删正好在第 0 片叶之外的号（逼根层级来回 0 ↔ 1）：每次写、每次删重写几个节点。"""
    planner = Planner(g, rule)
    version = planner.initial()
    version = planner.step(version, 1, "write")[0]
    edge = g.U
    writes = deletes = 0
    for _ in range(rounds):
        version, rewritten, _, violations = planner.step(version, edge, "write")
        assert not violations, violations
        writes += len(rewritten)
        version, rewritten, _, violations = planner.step(version, edge, "delete")
        assert not violations, violations
        deletes += len(rewritten)
    return "rounds=%d rewritten_by_the_writes=%d rewritten_by_the_deletes=%d final_root_level=%d" % (
        rounds, writes, deletes, version.root_level)



def terminal_demo(g, rule, files):
    """files 里每个号先写进 S，再按给定次序逐个删到空：每一步交回新位置集合、根层级、重写集、换下集。"""
    planner = Planner(g, rule)
    version = planner.initial()
    for inode in files:
        version = planner.step(version, inode, "write")[0]
    return planner, version


def terminal_lines(g, rule, files, deletes):
    planner, version = terminal_demo(g, rule, files)
    out = ["start S=%s positions=%s level=%d" % (sorted(version.entries), sorted(version.positions), version.root_level)]
    for inode in deletes:
        version, rewritten, replaced, violations = planner.step(version, inode, "delete")
        assert not violations, violations
        out.append("delete %d -> S=%s positions=%s level=%d rewritten=%s replaced=%s" % (
            inode, sorted(version.entries), sorted(version.positions), version.root_level,
            sorted(rewritten), sorted(replaced)))
    return out



def main(argv):
    seeds = range(1, 41)
    steps = 120
    selftest = "--selftest" in argv
    if "--seed" in argv:
        seeds = [int(argv[argv.index("--seed") + 1])]
    if "--steps" in argv:
        steps = int(argv[argv.index("--steps") + 1])
    small = Geometry(3, 2)       # 小几何：层级常变
    real = Geometry(143, 147)    # 产品几何（format 常量 EXTENT_TREE_UPPER_LEAF_INODES、EXTENT_TREE_INTERNAL_FANOUT）
    results = {}
    for name, g, universe in (("small_U3_F2", small, 60), ("real_U143_F147", real, 143 * 147 * 3)):
        for rule in ("A", "B"):
            counters, problems = survey(g, rule, None, seeds, steps, universe)
            results[(name, rule)] = (counters, problems)
            print("name=survey geometry=%s rule=%s %s" % (
                name, rule, " ".join("%s=%d" % kv for kv in counters.items())))
            kinds = sorted({p[0] for p in problems})
            print("name=survey_problems geometry=%s rule=%s kinds=%s" % (name, rule, ",".join(kinds) or "none"))
    for rule in ("A", "B"):
        print("name=boundary_churn rule=%s %s" % (rule, boundary_churn(real, rule, 100)))
    for deletes in ((1, 500), (500, 1)):
        for line in terminal_lines(real, "A", [1, 500], deletes):
            print("name=terminal rule=A order=%s %s" % ("-".join(map(str, deletes)), line))
    same_shape, ids_differ, trail_a, trail_b = order_pair_demo(real, "A")
    print("name=order_pair rule=A final_positions_equal=%s final_birth_ids_differ=%s" % (same_shape, ids_differ))
    _, _, trail_b1, trail_b2 = order_pair_demo(real, "B")
    print("name=order_pair_intermediate_B first_order_after_first_delete=%s second_order_after_first_delete=%s" % (
        trail_b1[0], trail_b2[0]))
    print("name=order_pair_intermediate_A first_order_after_first_delete=%s second_order_after_first_delete=%s" % (
        trail_a[0], trail_b[0]))
    bad = []
    for total, kept in ((300, 200), (300, 144), (300, 1), (21168 + 5, 21168), (21168 + 5, 144), (20000, 145), (145, 144)):
        ok, diff = lower_batch_check(144, 147, total, kept)
        print("name=lower_batch units=%d kept=%d path_rule_equals_content_rule=%s" % (total, kept, ok))
        if not ok:
            bad.append((total, kept, diff))
    # 小几何上扫全部 (total, kept)
    exhaustive = 0
    exhaustive_bad = 0
    for total in range(2, 40):
        for kept in range(1, total):
            exhaustive += 1
            if not lower_batch_check(3, 2, total, kept)[0]:
                exhaustive_bad += 1
    print("name=lower_batch_exhaustive_small leaf_units=3 fanout=2 cases=%d disagreements=%d" % (exhaustive, exhaustive_bad))
    if selftest:
        failed = 0
        for mutant in ("keep-empty-leaf", "no-ancestor-rewrite", "shrink-keeps-old-root"):
            counters, problems = survey(small, "A", mutant, seeds, steps, 60)
            kinds = sorted({p[0] for p in problems})
            red = bool(problems)
            print("name=selftest mutant=%s red=%s kinds=%s" % (mutant, red, ",".join(kinds)))
            if not red:
                failed += 1
        correct = results[("small_U3_F2", "A")][1]
        print("name=selftest mutant=none red=%s" % bool(correct))
        if correct:
            failed += 1
        return 1 if failed else 0
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
