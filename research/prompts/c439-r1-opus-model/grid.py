#!/usr/bin/env python3
"""把 sweep 的 results.txt 合成判定格：只看「续作里至少跑过一次门禁」的那些序列。
守规矩的续作：每一次提交（c 或 k）紧跟在一次门禁之后（S 是 gate-staged，G 是带 GATE_BASE=HEAD 的 gate-staged，N 是「跑全量工作区」那条路，
`.claude/skills/gate/staged-gate.md` 第 16 行两条都许）；前缀以一次门禁收尾的历史（PREFIX_ENDS_WITH_GATE），续作开头的提交也算守规矩。
用法：grid.py <results.txt>…
每格打一行：name=grid hist=… cand=… gated=<至少跑过一次门禁的续作条数> gated_never=<其中一次都没判到的> disciplined=<守规矩且跑过门禁的条数> disciplined_never=<其中一次都没判到的> verdict=<permanent|some|never-missed> example=<最短的一条守规矩而漏判的续作，没有就取最短的漏判续作，都没有写 none>
  permanent：守规矩且跑过门禁的续作全部漏判；some：有的漏有的不漏；never-missed：没有一条漏判
  gated_judged_after_prefix：跑过门禁的续作里，被追踪的那一行在前缀之后又进了窗口的条数（h14 用它数「别的会话判绿过的行又被算进来」）
"""
import collections, sys
rows = collections.defaultdict(list)
for path in sys.argv[1:]:
    for line in open(path, encoding='utf-8'):
        if line.startswith('result '):
            f = dict(p.split('=', 1) for p in line.split()[1:] if '=' in p)
            rows[(f['hist'], f['cand'])].append(f)
def gated(c): return c != 'none' and any(x in 'NSG' for x in c)
PREFIX_ENDS_WITH_GATE = {'h1_c439', 'h9_aba', 'h9_stash', 'h9_commit_undo', 'h15_override'}
def disciplined(c, hist):
    if c == 'none': return False
    return all(c[i] not in 'ck' or (c[i-1] in 'NSG' if i > 0 else hist in PREFIX_ENDS_WITH_GATE) for i in range(len(c)))
key = lambda c: (len(c), c)
for (hist, cand), items in sorted(rows.items()):
    assert all(f.get('rc') == '0' for f in items), (hist, cand)
    g = [f for f in items if gated(f['cont'])]
    gn = [f for f in g if f['judged_ever'] == 'no']
    d = [f for f in g if disciplined(f['cont'], hist)]
    rejudged = sum(1 for f in g if f.get('after_prefix', 'none') != 'none')
    dn = [f for f in d if f['judged_ever'] == 'no']
    verdict = 'permanent' if d and len(dn) == len(d) else ('never-missed' if not gn else 'some')
    example = min((f['cont'] for f in dn), key=key, default=None) or min((f['cont'] for f in gn), key=key, default='none')
    print(f'name=grid hist={hist} cand={cand} gated={len(g)} gated_never={len(gn)} disciplined={len(d)} disciplined_never={len(dn)} verdict={verdict} example={example} gated_judged_after_prefix={rejudged}')
