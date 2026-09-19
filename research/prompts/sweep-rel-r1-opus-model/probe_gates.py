#!/usr/bin/env python3
"""sweep-rel-r1 攻方腿（Opus）：T6（整句照抄一条被这一阶段改掉的旧句）+ 三道候补闸在 T0–T6 上各红不红。
候补闸只在这份模型里实现，没进 stale-candidates.py，被攻过零轮。
用法：cd 仓根 && python3 research/prompts/sweep-rel-r1-opus-model/probe_gates.py 草稿目录（attack_facts.py 先跑过，T*.tsv 在里面）"""
import importlib.util, os, re, sys
draft = sys.argv[1]
spec = importlib.util.spec_from_file_location('sc', 'research/scripts/stale-candidates.py')
sc = importlib.util.module_from_spec(spec); spec.loader.exec_module(sc)
ROOT = os.getcwd(); BASE, TARGET = sc.BENCHMARK_BASE, sc.BENCHMARK_TARGET
base_corpus = sc.current_state_corpus(ROOT, BASE)
target_corpus = sc.current_state_corpus(ROOT, TARGET)
target_lines = {l for ls in target_corpus.values() for _, l in ls}
entries = {e: h for e, _, h in sc.new_change_entries(ROOT, BASE, TARGET)}
def hits(term, corpus):
    parts = sc.compile_search_term(term)
    return [l for ls in corpus.values() for _, l in ls if sc.search_term_matches(parts, l)]
def real(f):
    return not f['旧事实'].startswith((sc.REWORDING_ONLY_OLD_FACT_PREFIX, sc.NEWLY_ADDED_OLD_FACT_PREFIX))

# T6：每行实事的检索词换成「基准里命中原检索词、而结束那一版已不在」的一整行（re.escape）
bench = sc.read_fact_table(sc.BENCHMARK_FACTS)
HEAD = '\t'.join(sc.FACT_TABLE_COLUMNS) + '\n'
rows, n6 = [], 0
for f in bench:
    r = [f[c] for c in sc.FACT_TABLE_COLUMNS]
    if real(f):
        gone = [l for l in hits(f['检索词'], base_corpus) if l not in target_lines and len(l.strip()) >= 8]
        if gone:
            r[3] = re.escape(gone[0].strip()); n6 += 1
    rows.append(r)
with open(os.path.join(draft, 'T6.tsv'), 'w', encoding='utf-8') as fh:
    fh.write(HEAD + ''.join('\t'.join(r) + '\n' for r in rows))
cands = sc.build_candidates(ROOT, TARGET, sc.read_fact_table(os.path.join(draft, 'T6.tsv')))
hs = {sc.location_hash(c[3]) for c in cands}
cov = sum(1 for h in sc.BENCHMARK_KNOWN_STALE_LOCATION_HASHES if h in hs)
import io, contextlib
buf = io.StringIO()
with contextlib.redirect_stdout(buf):
    code = sc.check_facts(ROOT, BASE, TARGET, os.path.join(draft, 'T6.tsv'))
print(f'T6 整句照抄被改掉的旧句：换了 {n6} 行；--check-facts 退出码 {code}；罩住 {cov}（下限 20）；候选 {len(cands)} 行')
print()

WORDING = re.compile(r'措辞|指名|路径|搬|一个字没改|一个字不改|一个字没动|一个字不动|原样')
def gate_a(facts):
    """只改措辞行引的每条 H，标题里要有改措辞的字样。"""
    bad = []
    for f in facts:
        if f['旧事实'].startswith(sc.REWORDING_ONLY_OLD_FACT_PREFIX):
            for h in re.findall(r'H\d+', f['出处']):
                if h in entries and not WORDING.search(entries[h]):
                    bad.append(f'{f["编号"]}:{h}')
    return bad
def gate_b(facts):
    """带 && 的检索词，拿掉 && 之后的那几段、第一段自己在结束那一版要超过 150 行（真的太宽才许收窄）。"""
    return [f['编号'] for f in facts if real(f) and sc.TERM_CONJUNCTION in f['检索词']
            and len(hits(f['检索词'].split(sc.TERM_CONJUNCTION)[0], target_corpus)) <= sc.MAXIMUM_CANDIDATE_LINES_PER_FACT]
def gate_c(facts):
    """检索词在基准命中的行里，至少一行在结束那一版原样还在（不许只罩被这一阶段改掉的旧句）。"""
    return [f['编号'] for f in facts if real(f) and not any(l in target_lines for l in hits(f['检索词'], base_corpus))]

print('| 表 | 闸 a（只改措辞行引的 H 标题要有改措辞字样）红的行 | 闸 b（&& 只许在第一段超 150 行时用）红的行 | 闸 c（基准命中的行至少一行结束那一版还在）红的行 |')
print('|---|---|---|---|')
for name in ['T0', 'T1', 'T1b', 'T3', 'T4', 'T5', 'T6']:
    facts = sc.read_fact_table(os.path.join(draft, name + '.tsv'))
    a, b, c = gate_a(facts), gate_b(facts), gate_c(facts)
    fmt = lambda xs: f'{len(xs)}' + (f'（{"、".join(xs[:12])}{"…" if len(xs) > 12 else ""}）' if xs else '')
    print(f'| {name} | {fmt(a)} | {fmt(b)} | {fmt(c)} |')
