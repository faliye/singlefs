#!/usr/bin/env python3
"""用同一张事实表给 stale-candidates.py 载体模式罩不到的三类文档出候选（这一轮治理的绕过，不进仓）。
用法：python3 extra-candidates.py 事实表.tsv 候选.tsv
"""
import importlib.util, sys
sys.dont_write_bytecode = True
ROOT = '/home/fy5090/code/singlefs'
fact_path, out_path = sys.argv[1], sys.argv[2]
spec = importlib.util.spec_from_file_location('stale', f'{ROOT}/scripts/kb/stale-candidates.py')
stale = importlib.util.module_from_spec(spec)
spec.loader.exec_module(stale)
stale.CARRIER_PATTERNS[:] = [
    r'^crates/singlefs-checker/invariants/[^/]+\.md$',
    r'^\.claude/skills/[^/]+/(?!SKILL\.md$)[^/]+\.md$',
    r'^scripts/(?:[^/]+/)*README\.md$',
]
facts = stale.read_fact_table(fact_path)
candidates = stale.build_candidates(ROOT, None, facts)
with open(out_path, 'w', encoding='utf-8') as handle:
    stale.write_candidates(candidates, handle)
carriers = sorted({location.rsplit(':', 1)[0] for _, _, _, location, _ in candidates})
print(f'  ✓ 额外候选 {out_path}：{len(candidates)} 行，落在 {len(carriers)} 份载体；载体集合 {len(stale.current_state_corpus(ROOT, None))} 份')
