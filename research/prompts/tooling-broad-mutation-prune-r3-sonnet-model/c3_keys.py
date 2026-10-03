"""C3 的键：(文件, 函数, 函数内相对位置, 替换文) 在 HEAD 清单 5091 条上唯一不唯一；函数名重名的多不多。只读 list.json 与 rows.json。"""
import json, re, collections, sys
sys.path.insert(0, '.')
L = json.load(open('/home/fy5090/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4/list.json'))
rows = json.load(open('rows.json'))
assert len(L) == len(rows)
def qualified(name):
    m = re.match(r'^[^:]+:\d+:\d+: (.*)$', name)
    d = m.group(1)
    m2 = re.search(r' in (.+)$', d)
    if m2: return m2.group(1)
    m3 = re.match(r'^replace (.+?)(?: -> .+?)? with ', d)
    return m3.group(1) if m3 else None
q = [qualified(e['name']) for e in L]
print('qualified name not parsed:', sum(1 for x in q if x is None))
# 重名：同文件同限定名对应多个函数起点（rows.json 的 fnstart）
starts = collections.defaultdict(set); cnt = collections.Counter()
for e, r, qq in zip(L, rows, q):
    if r['fnstart'] is not None:
        starts[(e['file'], qq)].add(r['fnstart']); cnt[(e['file'], qq)] += 1
dups = {k: v for k, v in starts.items() if len(v) > 1}
print('(file, qualified) keys', len(starts), 'with >1 function start:', len(dups), 'mutants in them:', sum(cnt[k] for k in dups))
bare = collections.defaultdict(set); bc = collections.Counter()
for r in rows:
    if r['fn']: bare[(r['file'], r['fn'])].add(r['fnstart']); bc[(r['file'], r['fn'])] += 1
bd = {k: v for k, v in bare.items() if len(v) > 1}
print('(file, bare fn name) keys', len(bare), 'with >1 function start:', len(bd), 'mutants in them:', sum(bc[k] for k in bd))
# 同一函数内：(相对行、列、替换文) 是否唯一；相对位置以函数起点行为零
import json as _j
from lex2 import src, line_starts
cache = {}
seen = collections.Counter(); total = 0
for e, r in zip(L, rows):
    if r['fnstart'] is None: continue
    f = e['file']
    if f not in cache:
        s = src(f); cache[f] = (s, line_starts(s))
    s, ls = cache[f]
    fl = s.count('\n', 0, r['fnstart']) + 1
    key = (f, r['fnstart'], e['start'][0] - fl, e['start'][1], e['end'][0] - fl, e['end'][1], e['replacement'])
    seen[key] += 1; total += 1
print('mutants in functions', total, 'distinct (fn, rel start, rel end, replacement) keys', len(seen), 'dup keys', sum(1 for v in seen.values() if v > 1))
