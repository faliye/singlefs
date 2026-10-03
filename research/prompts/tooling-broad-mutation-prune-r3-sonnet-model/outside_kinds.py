"""C1 的「函数外面的部分变了 => 那个文件全部变异重判」是 14 个提交里条数的大头：把每个「外面变了」的 (提交, 文件) 按变了什么分类。
分类（近似，token 级）：test-mod（变化落在 #[cfg(test)] mod 里、函数之外）、use（use 语句）、type（struct/enum/type/trait/union）、impl（其余 impl/trait 块里函数之外）、const-static、other。
一个 (提交, 文件) 可以同时有几类。只读 git 与 rows.json。"""
import json, subprocess, collections, sys, difflib
sys.path.insert(0, '.')
from lex2 import tokens, fn_spans
R = '/home/fy5090/code/singlefs'
rows = json.load(open('rows.json'))
fileTotal = collections.Counter(r['file'] for r in rows)
res = json.load(open('c1_commits.json'))
def git(*a): return subprocess.run(['git', *a], capture_output=True, cwd=R).stdout.decode('utf8', errors='replace')
def test_spans(toks):
    out = []; n = len(toks)
    for k in range(n - 9):
        w = [t[1] for t in toks[k:k + 10]]
        if w[:7] == ['#', '[', 'cfg', '(', 'test', ')', ']'] and w[7] == 'mod' and w[9] == '{':
            d = 0; e = k + 9
            while e < n:
                if toks[e][0] == 'p':
                    if toks[e][1] == '{': d += 1
                    elif toks[e][1] == '}':
                        d -= 1
                        if d == 0: break
                e += 1
            out.append((toks[k][2], toks[min(e, n - 1)][3]))
    return out
def outside_tokens(s):
    toks = tokens(s); fs = fn_spans(s); ts = test_spans(toks)
    out = []
    last_kw = 'other'
    for k, x, a, b in toks:
        if any(f['hdr'] <= a < f['end'] for f in fs): continue
        if k == 'id' and x in ('use', 'struct', 'enum', 'type', 'trait', 'union', 'impl', 'const', 'static', 'mod', 'macro_rules'): last_kw = x
        if x == ';' or x == '}': pass
        in_test = any(p <= a < q for p, q in ts)
        out.append((x if k != 'str' else 'S', last_kw, in_test))
    return out
kinds_total = collections.Counter(); files_total = 0
rows_out = []
for r in res:
    c = r['c']
    allch = [f for f in git('diff', '--name-only', c + '^', c).split() if f in fileTotal]
    per = collections.Counter(); onlykind = collections.Counter()
    for f in allch:
        new = git('show', f'{c}:{f}'); old = git('show', f'{c}^:{f}')
        if not old: continue
        o, n = outside_tokens(old), outside_tokens(new)
        if [x[0] for x in o] == [x[0] for x in n]: continue
        sm = difflib.SequenceMatcher(None, [x[0] for x in o], [x[0] for x in n], autojunk=False)
        kinds = set()
        for tag, i1, i2, j1, j2 in sm.get_opcodes():
            if tag == 'equal': continue
            seg = n[j1:j2] or o[i1:i2]
            for tx, kw, in_test in seg:
                kinds.add('test-mod' if in_test else ('use' if kw == 'use' else ('type' if kw in ('struct', 'enum', 'type', 'trait', 'union') else ('impl' if kw == 'impl' else ('const-static' if kw in ('const', 'static') else 'other')))))
        for kd in kinds: per[kd] += 1
        key = '+'.join(sorted(kinds))
        onlykind[key] += fileTotal[f]
        files_total += 1
        for kd in kinds: kinds_total[kd] += 1
    rows_out.append((c, r['eff_s'], r['fn_s'], dict(per), dict(onlykind)))
for c, e, fo, per, ok in rows_out: print(c, 'eff', e, 'fn-only', fo, 'files-by-kind', per, 'mutants-by-kind-combo', ok)
print('outside-changed (commit,file) pairs', files_total, 'by kind', dict(kinds_total))
# 若「只有 test-mod 变」或「只有 use 变」的文件不算整份重判，14 个提交的条数会少多少
saved_testmod = sum(v for c, e, fo, per, ok in rows_out for k, v in ok.items() if k == 'test-mod')
saved_use = sum(v for c, e, fo, per, ok in rows_out for k, v in ok.items() if k == 'use')
saved_both = sum(v for c, e, fo, per, ok in rows_out for k, v in ok.items() if k in ('test-mod', 'use', 'test-mod+use'))
print('mutants in files whose outside change is test-mod only: %d ; use only: %d ; test-mod/use only combos: %d ; sum of eff over 14 commits: %d' % (saved_testmod, saved_use, saved_both, sum(e for c, e, fo, per, ok in rows_out)))
