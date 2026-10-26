#!/usr/bin/env python3
"""第三轮辩方腿 Z1：G7 改写「回退候选窗口移一格」的有界穷举。
状态是第二轮正推 gc_ring_proofs 的逻辑层：roots（旧到新的逻辑版本号）、floor、head（现行版本号）、window。
用户步：UP（现行换成一个没出现过的新版本）、CP（现行落成新根，候选超窗口挤最旧）、RB c（回退到窗口里内容为 c 的候选，没有则被拒）、RF c（F 抬到内容 c 第一次出现的位置）。
整理步 GC：只在现行内容 == 最后一个根的内容时插入，等于一次 CP（Relocate 不改逻辑内容，第二轮正推 lemma_a_gc_checkpoint_only_shifts_the_window 的前提）。
同一串用户步在 A（不插 GC）和 B（任意位置插 GC）上各跑一遍，比较。
"""
import itertools, sys

def checkpoint(st):
    roots, floor, head, W = st
    roots = roots + [head]
    if len(roots) - floor > W:
        floor += 1
    return (roots, floor, head, W)

def window(st):
    roots, floor, _, _ = st
    return roots[floor:]

def step(st, op, fresh):
    roots, floor, head, W = st
    kind = op[0]
    if kind == 'UP':
        return (roots, floor, fresh, W), True
    if kind == 'CP':
        return checkpoint(st), True
    if kind == 'RB':
        c = op[1]
        w = window(st)
        if c not in w:
            return st, False
        return (roots, floor, c, W), True
    if kind == 'RF':
        c = op[1]
        w = window(st)
        if c not in w:
            return st, False
        return (roots, floor + w.index(c), head, W), True
    raise ValueError(kind)

def dedup(seq):
    out = []
    for x in seq:
        if not out or out[-1] != x:
            out.append(x)
    return out

def run(W, ops, gc_after):
    """ops: 用户步序列；gc_after: 一个集合，第 i 步之前插一次 GC（只在允许时）。返回 (最终状态, 被拒步下标集, GC 实际插入次数)。"""
    st = ([0], 0, 0, W)   # 版本 0 已是根，现行 == 0
    fresh = 1
    rejected = set()
    gcs = 0
    for i, op in enumerate(ops):
        if i in gc_after and st[2] == st[0][-1]:
            st = checkpoint(st); gcs += 1
        st, ok = step(st, op, fresh)
        if op[0] == 'UP':
            fresh += 1
        if not ok:
            rejected.add(i)
    if len(ops) in gc_after and st[2] == st[0][-1]:
        st = checkpoint(st); gcs += 1
    return st, rejected, gcs

def alphabet(versions):
    ops = [('UP',), ('CP',)]
    ops += [('RB', c) for c in range(versions)]
    ops += [('RF', c) for c in range(versions)]
    return ops

stats = dict(runs=0, same_acceptance=0, exact_shift_fail=0, subset_fail=0, loss_gt_k=0, head_diff=0, accepted_by_B_not_A=0, ceiling_fail=0)
examples = {}
LEN = int(sys.argv[1]) if len(sys.argv) > 1 else 6
import random
SAMPLE = sys.argv[2].split(':') if len(sys.argv) > 2 else None
for W in (1, 2, 3):
    ab = alphabet(4)
    for n in range(1, LEN + 1):
        if SAMPLE:
            rng = random.Random(int(SAMPLE[2]) + W * 1000 + n)
            ops_iter = (tuple(rng.choice(ab) for _ in range(n)) for _ in range(int(SAMPLE[1])))
        else:
            ops_iter = itertools.product(ab, repeat=n)
        for ops in ops_iter:
            A, rejA, _ = run(W, ops, set())
            for mask in (range(1, 1 << (n + 1)) if not SAMPLE else [rng.randrange(1, 1 << (n + 1)) for _ in range(8)]):
                gc_after = {i for i in range(n + 1) if mask >> i & 1}
                B, rejB, k = run(W, ops, gc_after)
                if k == 0:
                    continue
                stats['runs'] += 1
                if rejA != rejB:
                    # 接受情况分叉之后两边不再是同一串用户历史，只记是谁多接受了一步
                    if rejA - rejB:
                        stats['accepted_by_B_not_A'] += 1
                        examples.setdefault('accepted_by_B_not_A', (W, ops, sorted(gc_after)))
                    continue
                stats['same_acceptance'] += 1
                wa, wb = window(A), window(B)
                if A[2] != B[2]:
                    stats['head_diff'] += 1
                # 读法一（第二轮正推 5.4 的字面）：整理那份的窗口 = 不整理那份的窗口再多移 k 格（去掉相邻重复的整理根）
                if not any(dedup(wb) == dedup(wa)[j:] for j in range(0, k + 1)):
                    stats['exact_shift_fail'] += 1
                    examples.setdefault('exact_shift_fail', (W, ops, sorted(gc_after), wa, wb))
                # 读法二：内容集合 B ⊆ A，且少掉的至多 k 个
                if not set(wb) <= set(wa):
                    stats['subset_fail'] += 1
                    examples.setdefault('subset_fail', (W, ops, sorted(gc_after), wa, wb))
                if len(set(wa) - set(wb)) > k:
                    stats['loss_gt_k'] += 1
                    examples.setdefault('loss_gt_k', (W, ops, sorted(gc_after), wa, wb))
                if (len(B[0]) - 1) - (len(A[0]) - 1) != k:
                    stats['ceiling_fail'] += 1
print('name=g7_window_shift', ' '.join(f'{k}={v}' for k, v in stats.items()))
for k, v in examples.items():
    print('name=g7_example', k, v)
