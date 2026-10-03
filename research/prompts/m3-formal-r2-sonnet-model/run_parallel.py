#!/usr/bin/env python3
"""把 reader_rule_model 的 A 域枚举按第一个槽的 73 种状态切开，用至多 4 个进程并行（线程上限 4），再把各分项相加。
用法：python3 run_parallel.py <n> [monotone]
"""
import sys
from itertools import product
from multiprocessing import Pool
import reader_rule_model as m

KEYS = ['states', 'failed', 'applied_ge1', 'as_whole', 'as_cont', 'as_stops', 'd_wc', 'd_ws', 'd_cs',
        'j6', 'a_struct', 'rejected', 'base', 'mono_states', 'base_in_q']


def q_holds(table):
    """实现与 S 连着看读法不同的那批态的成因假设：所选根自己那次发布的末条（同 txg 带标志）的计数器 f，
    表里有 txg 大于所选根、计数器小于 f 的记录——同实例里 txg 随计数器回退"""
    flagged = [r for r in table.values() if r[1] == m.T and r[3]]
    if len(flagged) != 1:
        return False
    f = flagged[0][0]
    return any(r[1] > m.T and r[0] < f for r in table.values())


def work(args):
    n, monotone_only, first = args
    cnt = {k: 0 for k in KEYS}
    for anchor in (None, m.ANCHOR_RECORD):
        for rest in product(m.SLOT_STATES, repeat=n - 1):
            slots = (first,) + rest
            table = m.build_table(anchor, slots)
            mono = m.is_monotone(table)
            if monotone_only and not mono:
                continue
            cnt['states'] += 1
            if any(s is not None and s[1] == 0 for s in slots):
                cnt['rejected'] += 1
            r = m.impl(table)
            if r[0] == 'err':
                cnt['failed'] += 1
            elif r[1] != m.CHOSEN:
                cnt['applied_ge1'] += 1
            sw, sc, ss = m.spec(table, 'whole'), m.spec(table, 'cont'), m.spec(table, 'stops')
            cnt['as_whole'] += r != sw
            cnt['as_cont'] += r != sc
            cnt['as_stops'] += r != ss
            cnt['d_wc'] += sw != sc
            cnt['d_ws'] += sw != ss
            cnt['d_cs'] += sc != ss
            if r != sc:
                cnt['base'] += 1
                cnt['base_in_q'] += q_holds(table)
            if r[0] == 'ok':
                cnt['j6'] += bool(m.j6_violation(table, r))
                cnt['a_struct'] += (not m.a_structure_ok(r, table))
            cnt['mono_states'] += mono
    return cnt


if __name__ == '__main__':
    n = int(sys.argv[1])
    monotone_only = len(sys.argv) > 2 and sys.argv[2] == 'monotone'
    with Pool(4) as pool:
        parts = pool.map(work, [(n, monotone_only, f) for f in m.SLOT_STATES])
    total = {k: sum(p[k] for p in parts) for k in KEYS}
    print('n=%d monotone_only=%s' % (n, monotone_only), ' '.join('%s=%d' % kv for kv in total.items()))
