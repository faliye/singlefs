#!/usr/bin/env python3
"""正推腿的小原型：把 crates/singlefs-core/src/recovery.rs 的 replay_journal（3057-3229）及其三个辅助函数
逐分支翻成 Python，点名单元校验与反向链重读各抽成一个布尔（CRC 当未解释函数），
然后在小界内穷举、拿三条性质核（M2 句法、M2 语义在合法历史上、M3 单调在合法历史上）。
这是「翻译的模型」，不是 Rust 本身：它证不了 Rust 满足性质，只量两样东西——翻这一截要多少行、界取多大时穷举规模怎么长。
只读、纯 Python、无外部依赖。"""
import itertools
import sys
from collections import namedtuple

sys.dont_write_bytecode = True

# 一条记录（只留 replay_journal 读的字段；counter 就是 jsn 计数器）
Rec = namedtuple("Rec", "instance counter txg txn commit ordinal last chain verify")
# chain: 'H' 前一条在盘上且链值对 / 'B' 前一条在盘上而链值不等；前一条不在 records 里自动是「不在盘上」
# verify: 该记录点名单元全部读得出且校验和对


class Err(Exception):
    pass


def replay(root, records, limit, verify_named_units=True, flag_any=False):
    """root=(instance, txg)；records: dict[(instance, counter)] -> Rec。
    返回 (prefix_applied, applied_records, effective_root)；等价 recovery.rs 的 replay_journal。"""
    root_inst, root_txg = root
    water = (root_inst, root_txg)
    above = sorted((r for r in records.values()
                    if r.instance == root_inst and (r.instance, r.txg) > water),
                   key=lambda r: (r.instance, r.counter))                        # recovery.rs:3076-3082
    # counter_of_the_last_record_the_root_covers（2962-2987）
    flagged = [r.counter for r in records.values()
               if r.instance == root_inst and r.txg == root_txg and r.last]
    if len(flagged) > 1:
        raise Err("more than one last flag")
    anchor = flagged[0] if flagged else None
    expected = (root_inst, anchor + 1) if anchor is not None else None            # 3094-3096
    chain_start_txg = root_txg + 1                                                  # 3099-3106（u64 溢出略）
    open_publish = []
    applied = []
    eff = root
    for r in above[:limit]:                                                         # 3109
        if expected is not None:
            if (r.instance, r.counter) != expected:
                break
        elif r.txg != chain_start_txg or r.ordinal != 1:
            break
        prev = records.get((r.instance, r.counter - 1)) if r.counter - 1 >= 1 else None
        if prev is not None and r.chain == 'B':                                    # 3121-3126
            break
        if not open_publish and r.ordinal != 1:                                    # 3130-3134
            break
        if open_publish:
            p = open_publish[-1]
            if r.txg != p.txg:
                break                                                              # 3138
            if r.ordinal != p.ordinal + 1:
                break                                                              # 3143
            if not p.commit and r.txn != p.txn:
                break                                                              # 3152
        expected = (r.instance, r.counter + 1)                                     # 3158
        if not r.commit and r.last:
            break                                                                  # 3161
        if r.last and readable_same_publish_follows(r, records):                   # 3168
            break
        if verify_named_units and not r.verify:                                    # 3173-3191
            break
        open_publish.append(r)
        if not (r.last or flag_any):
            continue                                                               # 3200
        applied.extend(open_publish)
        open_publish = []
        eff = (r.instance, r.txg)                                                  # 3210-3226 只留 (instance, txg)
    return len(applied), applied, eff


def readable_same_publish_follows(r, records):
    """recovery.rs:2941-2953"""
    later = sorted((x for x in records.values() if x.instance == r.instance and x.counter >= r.counter + 1),
                   key=lambda x: x.counter)
    seen = False
    for x in later:
        if x.txg > r.txg:
            break
        if x.txg == r.txg:
            seen = True
    return seen


def whole_publishes(applied):
    """M2 句法：applied 恰是若干整次发布的拼接——同 txg 连续、序号 1..k、只有最后一条带末条标志。"""
    i = 0
    while i < len(applied):
        j = i
        ordinal = 1
        while True:
            if j >= len(applied):
                return False
            r = applied[j]
            if r.ordinal != ordinal or r.txg != applied[i].txg:
                return False
            if r.last:
                break
            ordinal += 1
            j += 1
        i = j + 1
    return True


def contiguous(applied):
    return all(b.counter == a.counter + 1 for a, b in zip(applied, applied[1:]))


# ---------- 三个检查 ----------
def check_unconstrained(n_slots, domains):
    """在任意记录表上核 M2 句法与 I-8.3 连续；同时数「施加序列里 txg 非单调」的表（无历史约束时的反例）。"""
    root = (1, 2)
    per = [None] + list(domains)
    bad_syn = bad_cont = nonmono = tables = 0
    example = None
    for combo in itertools.product(per, repeat=n_slots):
        recs = {}
        for c, d in enumerate(combo, 1):
            if d is not None:
                recs[(d[0], c)] = Rec(d[0], c, d[1], d[2], d[3], d[4], d[5], d[6], d[7])
        try:
            n, applied, eff = replay(root, recs, limit=n_slots)
        except Err:
            continue
        tables += 1
        if not whole_publishes(applied):
            bad_syn += 1
        if not contiguous(applied):
            bad_cont += 1
        seq = [r.txg for r in applied if r.last]
        if any(b < a for a, b in zip(seq, seq[1:])):
            nonmono += 1
            if example is None:
                example = sorted(recs.values())
    return tables, bad_syn, bad_cont, nonmono, example


def legal_history(sizes):
    """合法历史：根覆盖的发布 0（一条记录，counter=1，带末条标志，是锚点）之后接 len(sizes) 次发布，第 p 次 txg=2+p、
    sizes[p-1] 条记录、一次发布一个事务、只有最后一条带提交标记与末条标志；计数器全池接着走。"""
    root = (1, 2)
    recs = [Rec(1, 1, 2, 1, True, 1, True, 'H', True)]
    counter = 2
    for p, k in enumerate(sizes, 1):
        for o in range(1, k + 1):
            recs.append(Rec(1, counter, 2 + p, 1 + p, o == k, o, o == k, 'H', True))
            counter += 1
    return root, recs


def check_legal(sizes, flag_any=False):
    """在合法历史上枚举「哪几条记录落盘、落盘的哪几条点名单元验得过」的全部状态（每条记录三态：没落 / 落了但验不过 / 落了且验得过），核：
    (a) 施加恰是若干整次发布；(b) 施加 = 从锚点之后起、每次发布的记录都落盘且验得过的最长前缀；
    (c) 单调：c ⊆ c'（多落一条，或已落的一条多验过）⇒ 施加的记录集合 ⊆。⊆ 由单步生成，所以只核单步。"""
    root, full = legal_history(sizes)
    n = len(full)
    groups, idx = [], 1
    for k in sizes:
        groups.append(list(range(idx, idx + k)))
        idx += k
    results, viol_a, viol_b, viol_c = {}, 0, 0, 0
    for state in itertools.product((0, 1, 2), repeat=n):          # 0 没落 1 落了验不过 2 落了验得过
        recs = {(r.instance, r.counter): r._replace(verify=(s == 2))
                for r, s in zip(full, state) if s}
        _, applied, _ = replay(root, recs, limit=n, flag_any=flag_any)
        results[state] = frozenset(r.counter for r in applied)
        if not whole_publishes(applied):
            viol_a += 1
        exp = []
        for g in groups:
            if all(state[i] == 2 for i in g):
                exp.extend(full[i].counter for i in g)
            else:
                break
        if frozenset(exp) != results[state]:
            viol_b += 1
    for state, got in results.items():
        for i in range(n):
            if state[i] < 2:
                up = state[:i] + (state[i] + 1,) + state[i + 1:]
                if not got <= results[up]:
                    viol_c += 1
    return n, len(results), viol_a, viol_b, viol_c


if __name__ == "__main__":
    # 定义域：instance 固定 1；txg∈{2,3,4}（2 = 根的 txg，只有锚点那条会取到）；ordinal∈{1,2}；txn∈{1,2}；chain∈{H,B}
    dom = [(1, t, x, cm, o, l, ch, True)
           for t in (2, 3, 4) for x in (1, 2) for cm in (True, False)
           for o in (1, 2) for l in (True, False) for ch in ('H', 'B')]
    print("每个槽的取值数（含缺席）", len(dom) + 1)
    for n in (1, 2, 3):
        t, s, c, nm, ex = check_unconstrained(n, dom)
        print(f"任意记录表 N={n}: 表数={t} 句法违例={s} 连续违例={c} 施加序列 txg 非单调={nm}")
        if ex and nm:
            print("  第一个非单调例子（instance, counter, txg, txn, commit, ordinal, last, chain）：")
            for r in ex:
                print("   ", tuple(r)[:8])
    for sizes in [(1,), (2,), (1, 1), (1, 2), (2, 2), (1, 1, 1), (2, 1, 1), (1, 1, 1, 1), (2, 2, 2), (1, 1, 1, 1, 1, 1), (2, 2, 2, 2)]:
        n, k, a, b, c = check_legal(sizes)
        print(f"合法历史 各发布记录数={sizes}: 记录数(含锚点)={n} 落盘状态数={k} 违例 句法={a} 最长完整前缀={b} 单调={c}")
    print("变异（任意一条都算末条）：")
    for sizes in [(2,), (2, 2), (1, 2, 1)]:
        n, k, a, b, c = check_legal(sizes, flag_any=True)
        print(f"合法历史 各发布记录数={sizes}: 落盘状态数={k} 违例 句法={a} 最长完整前缀={b} 单调={c}")
