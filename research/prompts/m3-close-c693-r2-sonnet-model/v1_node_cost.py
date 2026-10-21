#!/usr/bin/env python3
"""正推腿 m3-close-c693-r2 V1：Carry 比 NoCarry 每次发布多写的 h_new x r_tree_dim 行，在 crates 的记账树几何下什么时候多写一个节点。

E186 的 11520 个取样点每格每代写行最多 47、记账树叶容量 477（crates/singlefs-core/src/transaction.rs 的 node_capacity_of_the_node_format 注释），
所以那片取样区里多写的行都落在同一片叶里。这份脚本把取样区扩到盘数 d = 1..4096：记账树每次发布整棵重写（crates/singlefs-core/src/code_two_tree.rs
plan_the_tree_after_this_publish 删上一版全部非台阶 key、插这一版的），写出的节点数 = 各层节点数之和。
行数：池级 3 + 每盘 6 x d + 台阶行 s 级（s 取 1..3）+ NoCarry 下不认识的头的行（0）/ Carry 下多 h_new x r_tree_dim 行。
几何：叶 477、内部 150（key 22 字节，crates 的值）；自证：叶容量算式 (16384-115-2*22)//(22+8+4) 必须等于 477，内部 (16384-115-2*22)//(22+86) 必须等于 150，不等退 3。
弄坏开关 V1_NODE_BREAK=entry_without_seq 把叶条目少算 seq 4 字节，自证必须红。
"""
import math, os, sys
NODE = 16384
BREAK = os.environ.get("V1_NODE_BREAK", "")
key = 22
header = 115 + 2 * key
leaf = (NODE - header) // (key + 8 + (0 if BREAK == "entry_without_seq" else 4))
inner = (NODE - header) // (key + 86)
ok = (leaf, inner) == (477, 150)
print(f"E7RESULT name=selftest_geometry leaf={leaf} inner={inner} expected=477/150 equal={int(ok)}")


def nodes(rows):
    level = [max(1, math.ceil(rows / leaf))]
    while level[-1] > 1:
        level.append(math.ceil(level[-1] / inner))
    return sum(level), len(level)


for r_tree_dim in (1, 2):
    for h_new in (1, 2, 4):
        extra = h_new * r_tree_dim
        differ = []
        total = 0
        for d in range(1, 4097):
            for s in (1, 2, 3):
                total += 1
                base = 3 + 6 * d + s
                n0, _ = nodes(base)
                n1, _ = nodes(base + extra)
                if n1 > n0:
                    differ.append((d, s))
        ds = sorted({d for d, _ in differ})
        print(f"E7RESULT name=carry_writes_more_nodes r_tree_dim={r_tree_dim} h_new={h_new} extra_rows={extra} points={total} differing={len(differ)} permille={1000 * len(differ) // total} first_devices={ds[:6]}")
print(f"E7RESULT name=done selftest_ok={int(ok)}")
sys.exit(0 if ok else 3)
