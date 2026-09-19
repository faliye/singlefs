#!/usr/bin/env python3
"""sweep-rel-r1 攻方腿（Opus）：150 行上限的代价——把盲写第四版里带 && 的检索词退回第一段（照定义是「太宽」、过不了闸），罩住的数变不变。
另数几个自然概念名词在结束那一版的命中行数。只数总数，不输出位置。
用法：cd 仓根 && python3 research/prompts/sweep-rel-r1-opus-model/probe_cap.py 草稿目录"""
import importlib.util, io, contextlib, os, sys
draft = sys.argv[1]
spec = importlib.util.spec_from_file_location('sc', 'research/scripts/stale-candidates.py')
sc = importlib.util.module_from_spec(spec); spec.loader.exec_module(sc)
ROOT = os.getcwd(); BASE, TARGET = sc.BENCHMARK_BASE, sc.BENCHMARK_TARGET
target_corpus = sc.current_state_corpus(ROOT, TARGET)
def th(term):
    parts = sc.compile_search_term(term)
    return sum(1 for ls in target_corpus.values() for _, l in ls if sc.search_term_matches(parts, l))
bench = sc.read_fact_table(sc.BENCHMARK_FACTS)
rows, widened = [], []
for f in bench:
    r = [f[c] for c in sc.FACT_TABLE_COLUMNS]
    if sc.TERM_CONJUNCTION in f['检索词'] and not f['旧事实'].startswith(('只改措辞', '新立：')):
        first = f['检索词'].split(sc.TERM_CONJUNCTION)[0]
        widened.append(f'{f["编号"]} {f["检索词"]} → {first}（{th(f["检索词"])} → {th(first)} 行）')
        r[3] = first
    rows.append(r)
path = os.path.join(draft, 'T7.tsv')
with open(path, 'w', encoding='utf-8') as fh:
    fh.write('\t'.join(sc.FACT_TABLE_COLUMNS) + '\n' + ''.join('\t'.join(r) + '\n' for r in rows))
buf = io.StringIO()
with contextlib.redirect_stdout(buf):
    code = sc.check_facts(ROOT, BASE, TARGET, path)
cands = sc.build_candidates(ROOT, TARGET, sc.read_fact_table(path))
hs = {sc.location_hash(c[3]) for c in cands}
print('放宽的行：'); print('\n'.join('  ' + w for w in widened))
print(f'T7 带 && 的实事行退回第一段：--check-facts 退出码 {code}；罩住 {sum(1 for h in sc.BENCHMARK_KNOWN_STALE_LOCATION_HASHES if h in hs)}（T0 为 20）；候选 {len(cands)} 行、{len({c[3] for c in cands})} 个不同的行')
print()
for term in ['层 0', '崩溃点重放', 'checker', '门禁 54 号', '树表', '树表条目', '第二个事务', '多次挂载', '回退', '释放', '覆盖写', '录制流', '暖机', '抬 F']:
    print(f'  {term}：结束那一版命中 {th(term)} 行')
