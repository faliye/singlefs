#!/usr/bin/env python3
"""用法：probe_compare.py <探针输出 W3COST 行文件>  把 planner.py 的复刻按同一套走法跑一遍，与探针里真规划器的 W3COST 行逐行比（打印的那些发布）。"""
import sys, re
sys.path.insert(0, __file__.rsplit("/", 1)[0])
from planner import Tree
STAT_TREE_NONE = 0; INODE_TREE = 65537; NO_DEVICE = 0xFFFFFFFF
def rows(gen):
    r = [(s, STAT_TREE_NONE, d, gen) for d in range(2) for s in (1, 2, 3, 5, 10, 11)]
    r += [(4, STAT_TREE_NONE, NO_DEVICE, gen), (6, STAT_TREE_NONE, NO_DEVICE, gen), (12, INODE_TREE, NO_DEVICE, gen)]
    return sorted(r)
def run(leaf, internal, publishes, change_every, label, print_every):
    t = Tree(leaf, internal); out = {}
    kept = False; prev_gen = 0; nkeys = 0
    for k in range(1, publishes + 1):
        gen = 2 + k; changed = change_every == 1 or k % change_every != 0
        deleted = []
        if k > 1:
            for row in rows(prev_gen):
                if row == (12, INODE_TREE, NO_DEVICE, prev_gen) and kept: continue
                deleted.append(row)
        inserted = rows(gen)
        nkeys += len(inserted) - len(deleted)
        rewritten = t.publish(sorted(deleted), sorted(inserted))
        if k <= 4 or k % print_every == 0 or k == publishes:
            nodes, leaves, height = t.stats()
            out[(label, k)] = f"rewritten={rewritten} height={height} nodes={nodes} leaves={leaves} keys={nkeys}"
        kept = changed; prev_gen = gen
    return out
def run_bulk(leaf, internal, prebuilt, extra, label):
    """先从空树一次装进「前 prebuilt 个发布都留了台阶行」那一刻的全部 key（升序逐把插），再逐发布走 extra 个（每发布都变，台阶行从不删）。"""
    t = Tree(leaf, internal); out = {}
    keys = [(12, INODE_TREE, NO_DEVICE, g) for g in range(3, prebuilt + 2)]
    prev_gen = 2 + prebuilt
    keys += rows(prev_gen)
    keys.sort()
    rewritten = t.publish([], keys)
    nodes, leaves, height = t.stats()
    nkeys = len(keys)
    out[(label, prebuilt)] = f"rewritten={rewritten} height={height} nodes={nodes} leaves={leaves} keys={nkeys} (bulk)"
    for k in range(prebuilt + 1, prebuilt + extra + 1):
        gen = 2 + k
        deleted = [r for r in rows(prev_gen) if r != (12, INODE_TREE, NO_DEVICE, prev_gen)]
        inserted = rows(gen)
        nkeys += len(inserted) - len(deleted)
        rewritten = t.publish(sorted(deleted), sorted(inserted))
        nodes, leaves, height = t.stats()
        out[(label, k)] = f"rewritten={rewritten} height={height} nodes={nodes} leaves={leaves} keys={nkeys}"
        prev_gen = gen
    return out

BULK = {"real-bulk-462": (477, 150, 462, 4), "real-bulk-36071": (477, 150, 36071, 5), "real-bulk-100000": (477, 150, 100000, 3), "real-bulk-2742507": (477, 150, 2742507, 5)}

if __name__ == "__main__":
    args = sys.argv[1:]
    if args and args[0] == "--red":
        # 自证会红：把复刻的「从中间切」弄坏成「切在 ⌊n÷2⌋」，对账必须判红
        import planner
        orig = planner.Tree._split
        def broken(self, n):
            if n.leaf:
                half = len(n.keys) // 2
                right = planner.Node(0, keys=n.keys[half:]); n.keys = n.keys[:half]; sep = right.keys[0]
            else:
                half = len(n.children) // 2
                right = planner.Node(n.level, children=n.children[half:]); n.children = n.children[:half]; sep = right.children[0][0]
            self._touch(right)
            return [sep, right]
        planner.Tree._split = broken
        args = args[1:]
        red_mode = True
    exp = {}
    red_mode = globals().get('red_mode', False)
    for line in (l for path in args for l in open(path, encoding="utf-8")):
        m = re.match(r"W3COST (\S+) leaf=(\d+) internal=(\d+) change_every=(\d+) publish=(\d+) (.*)", line.strip())
        if m: exp[(m[1], int(m[5]))] = (int(m[2]), int(m[3]), int(m[4]), m[6])
    cfgs = {"small-every-publish": (7, 4, 3000, 1, 1), "small-two-of-three": (10, 3, 3000, 3, 1), "tiny-every-publish": (5, 2, 3000, 1, 1),
            "real-every-publish": (477, 150, 6000, 1, 250), "real-two-of-three": (477, 150, 6000, 3, 250), "real-every-publish-40k": (477, 150, 40000, 1, 2000)}
    bad = 0; n = 0
    labels = sorted({k[0] for k in exp})
    for label in labels:
        if label in BULK:
            if red_mode: continue
            got = run_bulk(*BULK[label], label)
        else:
            leaf, internal, pubs, ce, pe = cfgs[label]
            if red_mode:
                if label != 'small-every-publish': continue
                pubs = 300   # 弄坏之后只跑前 300 个发布，够判红
            got = run(leaf, internal, pubs, ce, label, pe)
        for key, (l, i, c, text) in exp.items():
            if key[0] != label: continue
            n += 1
            if got.get(key) != text:
                bad += 1
                if bad <= 10: print("不同", key, "探针:", text, "复刻:", got.get(key))
        print(f"{label}: 比了 {sum(1 for k in exp if k[0]==label)} 行")
    print(f"probe-compare 比了 {n} 行，不同 {bad} 行")
    sys.exit(1 if bad else 0)
