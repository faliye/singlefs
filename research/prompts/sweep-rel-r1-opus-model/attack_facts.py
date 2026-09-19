#!/usr/bin/env python3
"""sweep-rel-r1 攻方腿（Opus）：A2 —— 过 --check-facts 却让腐烂进不了候选的事实表写法。
只读仓；生成的事实表写进 argv[1] 给的草稿目录。对照只数「罩住几处」，不输出任何位置、不拿哈希去对位置。
用法：cd 仓根 && python3 research/prompts/sweep-rel-r1-opus-model/attack_facts.py 草稿目录 [脚本路径]"""
import importlib.util, io, os, re, sys, contextlib

draft = sys.argv[1]
script = sys.argv[2] if len(sys.argv) > 2 else 'research/scripts/stale-candidates.py'
spec = importlib.util.spec_from_file_location('sc', script)
sc = importlib.util.module_from_spec(spec); spec.loader.exec_module(sc)
ROOT = os.getcwd(); BASE, TARGET = sc.BENCHMARK_BASE, sc.BENCHMARK_TARGET
HEAD = '\t'.join(sc.FACT_TABLE_COLUMNS) + '\n'

def check(path):
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        code = sc.check_facts(ROOT, BASE, TARGET, path)
    return code, buf.getvalue().strip().split('\n')[-1] if code == 0 else ' / '.join(l.strip() for l in buf.getvalue().split('\n') if '✗' in l)[:300]

def covered(path):
    cands = sc.build_candidates(ROOT, TARGET, sc.read_fact_table(path))
    hashes = {sc.location_hash(loc) for _, _, _, loc, _ in cands}
    return sum(1 for h in sc.BENCHMARK_KNOWN_STALE_LOCATION_HASHES if h in hashes), len(cands), len({c[3] for c in cands})

def write(name, rows):
    path = os.path.join(draft, name)
    with open(path, 'w', encoding='utf-8') as f:
        f.write(HEAD + ''.join('\t'.join(r) + '\n' for r in rows))
    return path

def real(fact):
    return not fact['旧事实'].startswith((sc.REWORDING_ONLY_OLD_FACT_PREFIX, sc.NEWLY_ADDED_OLD_FACT_PREFIX))

bench = sc.read_fact_table(sc.BENCHMARK_FACTS)
entries = sc.new_change_entries(ROOT, BASE, TARGET)
base_corpus = sc.current_state_corpus(ROOT, BASE)
def base_hits(term):
    parts = sc.compile_search_term(term)
    return sum(1 for ls in base_corpus.values() for _, l in ls if sc.search_term_matches(parts, l))

# 变更清单里的「旧：」「新：」片段（--changes 交给写表人的原料），按工具自己的 changed_segments 现算
segments = []
for p in sc.changed_paths(ROOT, BASE, TARGET):
    if sc.is_current_state_carrier(p):
        segments += sc.changed_segments(sc.read_version(ROOT, BASE, p), sc.read_version(ROOT, TARGET, p))

results = []
def run(label, name, rows, note=''):
    path = write(name, rows)
    code, line = check(path)
    cov, n, u = covered(path)
    results.append((label, name, len(rows), code, cov, n, u, note, line))

row = lambda f: [f[c] for c in sc.FACT_TABLE_COLUMNS]
run('T0 盲写第四版原样（对照）', 'T0.tsv', [row(f) for f in bench])
run('T1 一行「只改措辞」罩住全部 H', 'T1.tsv', [['F1', '只改措辞', '只改措辞', 'x', ';'.join(e for e, _, _ in entries)]])
run('T1b 原表每一行实事改标「只改措辞（…）」', 'T1b.tsv',
    [[f['编号'], ('只改措辞（' + f['旧事实'] + '）') if real(f) else f['旧事实'], f['新事实'], f['检索词'], f['出处']] for f in bench])

# T3：每行实事的检索词换成变更清单里一段被它罩住的「旧：」片段（re.escape），在基准必命中
t3, replaced = [], 0
for f in bench:
    r = row(f)
    if real(f):
        parts = sc.compile_search_term(f['检索词'])
        olds = [o for o, _ in segments if sc.search_term_matches(parts, o) and len(o) >= 6]
        if olds:
            r[3] = re.escape(max(olds, key=len)); replaced += 1
    t3.append(r)
run('T3 检索词照抄一段「旧：」片段', 'T3.tsv', t3, f'换了 {replaced} 行')

# T4：检索词原样，用 && 并上「那一段旧片段里的一个词」——只在被这一阶段改掉的旧句里同现
t4, replaced4 = [], 0
for f in bench:
    r = row(f)
    if real(f):
        parts = sc.compile_search_term(f['检索词'])
        olds = [o for o, _ in segments if sc.search_term_matches(parts, o)]
        for o in sorted(olds, key=len, reverse=True):
            words = [w for w in re.findall(r'[一-鿿]{2,4}', o) if not any(p.search(w) for p in parts)]
            for w in words:
                t = f['检索词'] + '&&' + re.escape(w)
                if base_hits(t) >= 1:
                    r[3] = t; replaced4 += 1; break
            if r[3] != f['检索词']:
                break
    t4.append(r)
run('T4 原检索词 && 旧片段里的一个词', 'T4.tsv', t4, f'收窄了 {replaced4} 行')

# T5：实事冒充「新立：」，检索词取一段「新：」片段里在基准零命中的串，新事实去掉六个禁词
t5, replaced5 = [], 0
for f in bench:
    r = row(f)
    if real(f):
        parts = sc.compile_search_term(f['检索词'])
        news = [n for o, n in segments if sc.search_term_matches(parts, o) and len(n) >= 4]
        for n in sorted(news, key=len, reverse=True):
            t = re.escape(n)
            if base_hits(t) == 0:
                r = [f['编号'], '新立：' + f['旧事实'], sc.NOT_NEWLY_ADDED_WORDS.sub('', f['新事实']), t, f['出处']]
                replaced5 += 1; break
    t5.append(r)
run('T5 实事冒充新立', 'T5.tsv', t5, f'冒充了 {replaced5} 行')

print('| 表 | 文件 | 行数 | --check-facts 退出码 | 罩住已知腐烂（下限 20） | 候选行 | 不同的行 | 备注 |')
print('|---|---|---|---|---|---|---|---|')
for label, name, rows, code, cov, n, u, note, line in results:
    print(f'| {label} | {name} | {rows} | {code} | {cov} | {n} | {u} | {note} |')
print()
for label, name, rows, code, cov, n, u, note, line in results:
    print(f'{name} --check-facts 末行：{line}')
