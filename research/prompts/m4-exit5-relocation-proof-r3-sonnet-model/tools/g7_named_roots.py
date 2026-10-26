#!/usr/bin/env python3
"""第三轮辩方腿 Z1：G7「回退候选窗口移一格」的有界穷举，第二版：用户按版本号（用户自己发布的根）点名，不按内容点名。
根环里的根是 (内容, 版本号)：用户的 CP 造一个新版本号，整理的发布（GC）造的根没有用户能点名的版本号（None）。
UP 换现行内容；CP 把现行落成新根（候选超窗口挤最旧）；RB u 回退到版本号 u 的根（不在窗口里被拒）；RF u 把 F 抬到版本号 u 的根（不在窗口里被拒）。
GC 只在现行内容 == 最后一个根的内容时插入。同一串用户步在 A（无 GC）与 B（任意位置插 GC）上各跑一遍；被拒的步各自当空操作，逐步比接受情况。
比的东西：现行内容；窗口里用户版本号的序列 U；B 接受而 A 拒绝的步；U_B 是不是 U_A 的后缀、少掉的是否至多 k 个。
"""
import itertools, random, sys

def cp(st, uid):
    roots, floor, head, W, nxt = st
    roots = roots + [(head, uid)]
    if len(roots) - floor > W:
        floor += 1
    return (roots, floor, head, W, nxt)

def window(st):
    return st[0][st[1]:]

def step(st, op, fresh):
    roots, floor, head, W, nxt = st
    k = op[0]
    if k == 'UP':
        return (roots, floor, fresh, W, nxt), True
    if k == 'CP':
        s2 = cp(st, nxt)
        return (s2[0], s2[1], s2[2], s2[3], nxt + 1), True
    w = window(st)
    uids = [u for _, u in w]
    if op[1] not in uids:
        return st, False
    i = uids.index(op[1])
    if k == 'RB':
        return (roots, floor, w[i][0], W, nxt), True
    if k == 'RF':
        return (roots, floor + i, head, W, nxt), True
    raise ValueError(k)

def run(W, ops, gc_before):
    st = ([(0, 0)], 0, 0, W, 1)
    fresh = 1; acc = []; gcs = 0
    for i, op in enumerate(ops):
        if i in gc_before and st[2] == st[0][-1][0]:
            st = cp(st, None); st = (st[0], st[1], st[2], st[3], st[4]); gcs += 1
        st, ok = step(st, op, fresh)
        if op[0] == 'UP':
            fresh += 1
        acc.append(ok)
    if len(ops) in gc_before and st[2] == st[0][-1][0]:
        st = cp(st, None); gcs += 1
    return st, acc, gcs

def user_ids(st):
    return [u for _, u in window(st) if u is not None]

def main():
    LEN = int(sys.argv[1]); SAMPLE = sys.argv[2].split(':') if len(sys.argv) > 2 else None
    S = dict(runs=0, b_accepts_a_rejects=0, a_accepts_b_rejects=0, head_differs=0, not_suffix=0, loss_gt_k=0, ceiling_diff_ne_k=0)
    ex = {}
    for W in (1, 2, 3):
        ab = [('UP',), ('CP',)] + [('RB', u) for u in range(4)] + [('RF', u) for u in range(4)]
        for n in range(1, LEN + 1):
            if SAMPLE:
                rng = random.Random(int(SAMPLE[2]) + W * 1000 + n)
                it = (tuple(rng.choice(ab) for _ in range(n)) for _ in range(int(SAMPLE[1])))
            else:
                it = itertools.product(ab, repeat=n)
            for ops in it:
                A, accA, _ = run(W, ops, set())
                masks = range(1, 1 << (n + 1)) if not SAMPLE else [rng.randrange(1, 1 << (n + 1)) for _ in range(8)]
                for mask in masks:
                    gc = {i for i in range(n + 1) if mask >> i & 1}
                    B, accB, k = run(W, ops, gc)
                    if k == 0:
                        continue
                    S['runs'] += 1
                    if any(b and not a for a, b in zip(accA, accB)):
                        S['b_accepts_a_rejects'] += 1; ex.setdefault('b_accepts_a_rejects', (W, ops, sorted(gc)))
                    if any(a and not b for a, b in zip(accA, accB)):
                        S['a_accepts_b_rejects'] += 1
                    if accA != accB:
                        continue   # 接受情况分叉之后不再是同一串用户历史
                    if A[2] != B[2]:
                        S['head_differs'] += 1; ex.setdefault('head_differs', (W, ops, sorted(gc)))
                    ua, ub = user_ids(A), user_ids(B)
                    if not any(ub == ua[j:] for j in range(len(ua) + 1)):
                        S['not_suffix'] += 1; ex.setdefault('not_suffix', (W, ops, sorted(gc), ua, ub))
                    elif len(ua) - len(ub) > k:
                        S['loss_gt_k'] += 1; ex.setdefault('loss_gt_k', (W, ops, sorted(gc), ua, ub))
                    if (len(B[0]) - len(A[0])) != k:
                        S['ceiling_diff_ne_k'] += 1
    print('name=g7_named_roots len=%d sample=%s' % (LEN, SAMPLE and ':'.join(SAMPLE)), ' '.join(f'{k}={v}' for k, v in S.items()))
    for k, v in ex.items():
        print('name=g7_named_roots_example', k, v)
main()
