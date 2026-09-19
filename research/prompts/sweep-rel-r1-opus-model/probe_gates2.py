#!/usr/bin/env python3
"""sweep-rel-r1 攻方腿（Opus）：再两道候补闸（c′、d）在 T0–T6 上各红不红；被攻过零轮。
用法：cd 仓根 && python3 research/prompts/sweep-rel-r1-opus-model/probe_gates2.py 草稿目录"""
import importlib.util, os, re, sys
draft = sys.argv[1]
spec = importlib.util.spec_from_file_location('sc', 'research/scripts/stale-candidates.py')
sc = importlib.util.module_from_spec(spec); spec.loader.exec_module(sc)
ROOT = os.getcwd(); BASE, TARGET = sc.BENCHMARK_BASE, sc.BENCHMARK_TARGET
target_corpus = sc.current_state_corpus(ROOT, TARGET)
entries = {e: h for e, _, h in sc.new_change_entries(ROOT, BASE, TARGET)}
def real(f):
    return not f['旧事实'].startswith((sc.REWORDING_ONLY_OLD_FACT_PREFIX, sc.NEWLY_ADDED_OLD_FACT_PREFIX))
def target_hits(term):
    parts = sc.compile_search_term(term)
    return sum(1 for ls in target_corpus.values() for _, l in ls if sc.search_term_matches(parts, l))
NEW = re.compile(r'新立|新开|新加|新增|加「|加并落地|建档')
def gate_c2(facts):
    """实事行的检索词在结束那一版至少命中一行（一行候选都不出的实事行判红）。"""
    return [f['编号'] for f in facts if real(f) and target_hits(f['检索词']) == 0]
def gate_d(facts):
    """新立行引的 H 里至少一条标题带新立字样；只引提交的不管。"""
    bad = []
    for f in facts:
        if f['旧事实'].startswith(sc.NEWLY_ADDED_OLD_FACT_PREFIX):
            hs = [h for h in re.findall(r'H\d+', f['出处']) if h in entries]
            if hs and not any(NEW.search(entries[h]) for h in hs):
                bad.append(f['编号'])
    return bad
print('| 表 | 闸 c′（实事行在结束那一版零命中）红的行 | 闸 d（新立行引的 H 标题没有新立字样）红的行 |')
print('|---|---|---|')
for name in ['T0', 'T1', 'T1b', 'T3', 'T4', 'T5', 'T6']:
    facts = sc.read_fact_table(os.path.join(draft, name + '.tsv'))
    c, d = gate_c2(facts), gate_d(facts)
    fmt = lambda xs: f'{len(xs)}' + (f'（{"、".join(xs[:14])}{"…" if len(xs) > 14 else ""}）' if xs else '')
    print(f'| {name} | {fmt(c)} | {fmt(d)} |')
