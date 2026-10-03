#!/usr/bin/env python3
"""m3-formal-r2 辩方腿：D23 已定项 4 读者规则第四格（带末条标志那条「之后」）与 E165 对抗域 A 的 Python 译本。
译本是读冻结副本 recovery.rs（replay_journal 第 3057 行起、readable_record_of_the_same_publish_follows 第 2941 行起）
与 e165-preregistration.md 5.2/5.3 写的，不是 Rust 本身。没有点名单元、没有盘数、没有环回绕：只翻 A 域（大环 64 槽、在飞上限 21）。
用法：python3 reader_rule_model.py <n>            全域（A 域 n 个槽）
      python3 reader_rule_model.py <n> monotone   只枚举「在的记录 txg 随计数器不减」的表
"""
import sys
from itertools import product

T = 100                      # 所选根的 txg
ANCHOR_COUNTER = 7
LIMIT = 21
CHOSEN = ('chosen',)

# 一条记录 = (counter, txg, ordinal, last, commit, txn)
SLOT_STATES = [None] + [
    (txg, od, last, commit, txn)
    for txg in (T, T + 1, T + 2) for od in (0, 1, 2) for last in (0, 1) for commit in (0, 1) for txn in (1, 2)
]
assert len(SLOT_STATES) == 73
ANCHOR_RECORD = (T, 1, 1, 1, 1)


def build_table(anchor, slots):
    """返回 counter -> 记录；序号 0 的记录 parse 拒收，进不了表"""
    table = {}
    if anchor is not None:
        table[ANCHOR_COUNTER] = (ANCHOR_COUNTER,) + anchor
    for i, s in enumerate(slots):
        if s is not None and s[1] != 0:
            c = ANCHOR_COUNTER + 1 + i
            table[c] = (c,) + s
    return table


def follows_impl(rec, table):
    """recovery.rs:2941 readable_record_of_the_same_publish_follows：计数器升序、txg 越过就停、见到同 txg 为真"""
    for c in sorted(table):
        if c <= rec[0]:
            continue
        later = table[c]
        if later[1] > rec[1]:
            return False
        if later[1] == rec[1]:
            return True
    return False


def follows_whole(rec, table):
    return any(c > rec[0] and table[c][1] == rec[1] for c in table)


def follows_stops(rec, table):
    c = rec[0] + 1
    while c in table:
        if table[c][1] > rec[1]:
            return False
        if table[c][1] == rec[1]:
            return True
        c += 1
    return False


def impl(table, t=T):
    """recovery.rs:3057 replay_journal（A 域：没有点名单元、反向链恒成立）"""
    flagged = [r for r in table.values() if r[1] == t and r[3]]
    if len(flagged) > 1:
        return ('err',)
    anchor = flagged[0][0] if flagged else None
    above = sorted([r for r in table.values() if r[1] > t], key=lambda r: r[0])
    expected = anchor + 1 if anchor is not None else None
    rebuilt = CHOSEN
    open_pub = []
    for r in above[:LIMIT]:
        c, txg, od, last, commit, txn = r
        if expected is not None:
            if c != expected:
                break
        elif txg != t + 1 or od != 1:
            break
        if not open_pub and od != 1:
            break
        if open_pub:
            p = open_pub[-1]
            if txg != p[1]:
                break
            if od != p[2] + 1:
                break
            if not p[4] and txn != p[5]:
                break
        expected = c + 1
        if not commit and last:
            break
        if last and follows_impl(r, table):
            break
        open_pub.append(r)
        if not last:
            continue
        rebuilt = ('rec', c, txg)
        open_pub = []
    return ('ok', rebuilt)


def spec(table, reading, t=T):
    """e165-preregistration.md 5.3 的 S（三种读法只在 (j) 上不同）"""
    follows = {'whole': follows_whole, 'cont': follows_impl, 'stops': follows_stops}[reading]
    flagged = [r for r in table.values() if r[1] == t and r[3]]
    if len(flagged) > 1:
        return ('err',)
    if flagged:
        head = flagged[0][0] + 1
    else:
        above = sorted([r for r in table.values() if r[1] > t], key=lambda r: r[0])
        if not above:
            return ('ok', CHOSEN)
        h = above[0]
        if h[2] != 1 or h[1] != t + 1:
            return ('ok', CHOSEN)
        head = h[0]
    out = CHOSEN
    open_pub = []
    cur = head
    taken = 0
    while True:
        r = table.get(cur)
        if r is None:                      # (a)
            break
        c, txg, od, last, commit, txn = r
        if not txg > t:                    # (b)
            break
        if taken >= LIMIT:                 # (c)
            break
        if not open_pub:
            if od != 1:                    # (f)
                break
        else:
            p = open_pub[-1]
            if txg != p[1] or od != p[2] + 1:   # (g)
                break
            if not p[4] and txn != p[5]:        # (h)
                break
        if last and not commit:            # (i)
            break
        if last and follows(r, table):     # (j)
            break
        taken += 1
        open_pub.append(r)
        if last:
            out = ('rec', c, txg)
            open_pub = []
        cur += 1
    return ('ok', out)


def is_monotone(table):
    txgs = [table[c][1] for c in sorted(table)]
    return all(a <= b for a, b in zip(txgs, txgs[1:]))


def a_structure_ok(res, table, t=T):
    if res[0] != 'ok' or res[1] == CHOSEN:
        return True
    _, c, txg = res[1]
    r = table[c]
    return txg > t and r[3] == 1 and r[4] == 1


def j6_violation(table, res1, t=T):
    if res1[0] != 'ok':
        return None                        # 第一次报错不计
    t2 = t if res1[1] == CHOSEN else res1[1][2]
    res2 = impl(table, t2)
    return not (res2[0] == 'ok' and res2[1] == CHOSEN)   # 第二次再施加了什么、或报错，都是违例


def states(n, monotone_only):
    for anchor in (None, ANCHOR_RECORD):
        for slots in product(SLOT_STATES, repeat=n):
            table = build_table(anchor, slots)
            if monotone_only and not is_monotone(table):
                continue
            # 序号 0 的记录不在表里，但状态数照数（E165 的状态数口径）
            yield anchor, slots, table


def main():
    n = int(sys.argv[1])
    monotone_only = len(sys.argv) > 2 and sys.argv[2] == 'monotone'
    cnt = dict(states=0, failed=0, applied_ge1=0, as_whole=0, as_cont=0, as_stops=0,
               d_wc=0, d_ws=0, d_cs=0, j6=0, a_struct=0, rejected=0,
               base_not_q=0, base=0, mono_states=0)
    for anchor, slots, table in states(n, monotone_only):
        cnt['states'] += 1
        if any(s is not None and s[1] == 0 for s in slots) or False:
            cnt['rejected'] += 1
        if anchor is not None and False:
            pass
        r = impl(table)
        if r[0] == 'err':
            cnt['failed'] += 1
        elif r[1] != CHOSEN:
            cnt['applied_ge1'] += 1
        sw, sc, ss = spec(table, 'whole'), spec(table, 'cont'), spec(table, 'stops')
        cnt['as_whole'] += r != sw
        cnt['as_cont'] += r != sc
        cnt['as_stops'] += r != ss
        cnt['d_wc'] += sw != sc
        cnt['d_ws'] += sw != ss
        cnt['d_cs'] += sc != ss
        if r != sc:
            cnt['base'] += 1
        if r[0] == 'ok':
            v = j6_violation(table, r)
            cnt['j6'] += bool(v)
            cnt['a_struct'] += (not a_structure_ok(r, table))
        if is_monotone(table):
            cnt['mono_states'] += 1
    print('n=%d monotone_only=%s' % (n, monotone_only), ' '.join('%s=%d' % kv for kv in cnt.items()))


if __name__ == '__main__':
    main()
