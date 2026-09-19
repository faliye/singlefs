#!/usr/bin/env python3
"""sweep-rel-r1 攻方腿（Opus）：两个改法在打中的格上量一次（副本上，被攻过零轮）。
F-cap：150 → 500，T0 与 T7 过不过 --check-facts；F-groups：区间倒写与组号不在候选表里时判红。
用法：cd 仓根 && python3 research/prompts/sweep-rel-r1-opus-model/probe_fixes.py 草稿目录"""
import importlib.util, io, contextlib, os, sys
draft = sys.argv[1]
def load():
    spec = importlib.util.spec_from_file_location('sc', 'research/scripts/stale-candidates.py')
    m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m); return m
sc = load()
def quiet(f, *a):
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        r = f(*a)
    return r, [l.strip() for l in buf.getvalue().split('\n') if l.strip()]
for cap in (150, 500):
    sc.MAXIMUM_CANDIDATE_LINES_PER_FACT = cap
    for t in ('T0', 'T7'):
        code, out = quiet(sc.check_facts, '.', sc.BENCHMARK_BASE, sc.BENCHMARK_TARGET, os.path.join(draft, t + '.tsv'))
        print(f'F-cap 上限 {cap}：{t} --check-facts 退出码 {code}；{out[-2] if code else out[-1]}'[:260])
# F-groups：改法写在这里（没进工具）：区间下界大于上界、或选中的组一个都不在候选表里，退出码 2
def parse_fixed(text, present):
    sel = sc.parse_group_ranges(text)
    for part in text.split(','):
        import re
        m = re.fullmatch(r'([A-Z])(\d+)(?:-[A-Z]?(\d+))?', part.strip())
        if m and m.group(3) and int(m.group(3)) < int(m.group(2)):
            return f'红：区间倒写 {part}'
    if not (sel & present):
        return '红：选中的组一个都不在候选表里'
    missing = sorted(sel - present)
    return f'过：选中 {len(sel & present)} 组' + (f'（{len(missing)} 个组号在候选表里没有：{"、".join(missing[:6])}）' if missing else '')
present = {k[0] for k in sc.read_candidate_keys(os.path.join(draft, 'cand.tsv'))}
for g in ('F6-F1', 'Z1', 'F1-F12', 'F1-G3'):
    print(f'F-groups --groups {g}：{parse_fixed(g, present)}')
