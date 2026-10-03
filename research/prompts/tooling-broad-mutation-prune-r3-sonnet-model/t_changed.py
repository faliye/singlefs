"""已判的 241 条抓到里，抓到它的那条测试 T（first_failing_test）在 14 个真实提交里「改了」的有多少，而变异所在的函数没改（C1 不重判、C3 沿用）。
T 的归属：「singlefs-core <模块>::tests::<名>」→ crates/singlefs-core/src/<模块>.rs 的函数 <名>；「singlefs-checker tests::<名>」→ checker/src/lib.rs；
「singlefs-harness::<二进制> <名>」→ crates/singlefs-harness/tests/<二进制>.rs 的函数 <名>。只读 git 与库目录。"""
import csv, json, os, re, subprocess, collections, hashlib, sys, statistics
sys.path.insert(0, '.')
from lex2 import tokens, fn_spans
R = '/home/fy5090/code/singlefs'
V = os.path.expanduser('~/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4')
rows = json.load(open('rows.json')); byseq = {r['seq']: r for r in rows}
last = {}
for r in csv.reader(open(V + '/results.tsv'), delimiter='\t'):
    if r and not r[0].startswith('#'): last[int(r[0])] = r
caught = {s: r for s, r in last.items() if r[3] == 'caught'}
def git(*a): return subprocess.run(['git', *a], capture_output=True, cwd=R).stdout.decode('utf8', errors='replace')
def t_target(T):
    m = re.match(r'singlefs-core (\w+)::tests::(\w+)$', T)
    if m: return f'crates/singlefs-core/src/{m.group(1)}.rs', m.group(2)
    m = re.match(r'singlefs-checker tests::(\w+)$', T)
    if m: return 'crates/singlefs-checker/src/lib.rs', m.group(1)
    m = re.match(r'singlefs-harness::(\S+) (\S+)$', T)
    if m: return f'crates/singlefs-harness/tests/{m.group(1)}.rs', m.group(2)
    return None, None
def fnhash(s):
    out = collections.defaultdict(list)
    for f in fn_spans(s):
        out[f['name']].append(hashlib.sha1(re.sub(r'\s+', '', s[f['hdr']:f['end']]).encode()).hexdigest())
    return out
def outside(s):
    fs = fn_spans(s)
    return ' '.join(s[a:b] for k, x, a, b in tokens(s) if not any(f['hdr'] <= a < f['end'] for f in fs))
res = json.load(open('c1_commits.json'))
cache = {}
def versions(c, f):
    k = (c, f)
    if k not in cache:
        new = git('show', f'{c}:{f}'); old = git('show', f'{c}^:{f}')
        cache[k] = (fnhash(old) if old else None, fnhash(new) if new else None, (outside(old) != outside(new)) if old and new else True, bool(old))
    return cache[k]
tot_carried = 0; tot_stale = 0; per = []
for r in res:
    c = r['c']; carried = 0; stale = 0; t_changed_any = set()
    for s, row in caught.items():
        m = byseq[s]; T = row[5]
        tf, tn = t_target(T)
        if tf is None: continue
        fo, fn_, outc, existed = versions(c, m['file'])
        if not existed or fo is None: continue          # 整份新文件：变异在 eff 里
        mf = m['fn']
        mut_changed = outc or (mf not in fo) or (mf not in fn_) or (fo[mf] != fn_[mf])
        # T 改没改：T 所在文件在这次提交里没动过就是没改
        allch = git('diff', '--name-only', c + '^', c).split() if ('ch', c) not in cache else cache[('ch', c)]
        cache[('ch', c)] = allch
        if tf not in allch: t_changed = False
        else:
            to, tn_, _, _ = versions(c, tf)
            t_changed = (to is None) or (tn not in to) or (tn_ is None) or (tn not in tn_) or (to[tn] != tn_[tn])
        if t_changed: t_changed_any.add(T)
        if not mut_changed:
            carried += 1
            if t_changed: stale += 1
    per.append((c, r['eff_s'], carried, stale, len(t_changed_any)))
    tot_carried += carried; tot_stale += stale
print('judged caught mutants:', len(caught))
print('commit eff_s | carried judged-caught mutants | of them whose T changed in this commit | distinct T changed')
for p in per: print(*p)
print('commits where >=1 carried judged-caught mutant lost/changed its catching test:', sum(1 for p in per if p[3] > 0), 'of', len(per))
print('sum carried', tot_carried, 'sum stale', tot_stale)
print('T class: core-lib %d, checker-lib %d, harness %d' % (sum(1 for r in caught.values() if r[5].startswith('singlefs-core ')), sum(1 for r in caught.values() if r[5].startswith('singlefs-checker ')), sum(1 for r in caught.values() if r[5].startswith('singlefs-harness'))))
