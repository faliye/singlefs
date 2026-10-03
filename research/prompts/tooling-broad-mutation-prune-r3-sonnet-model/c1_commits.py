"""C1 在 14 个真实提交上的数：改到的变异条数（两种「改没改」读法）、没改的变异里「别处改了」的几类暴露面、行号漂移。
只读 git（HEAD 是 fcc4d340）与 rows.json；不起 cargo、不起驱动。从本目录跑。
读法 s：函数文本去掉空白后逐字比（r2 辩方的 fnsig）；读法 t：词法 token 流比（注释与空白都不算，字符串字面量里的内容算）。"""
import json, subprocess, collections, hashlib, re, sys, statistics
sys.path.insert(0, '.')
from lex2 import tokens, fn_spans, line_starts
R = '/home/fy5090/code/singlefs'
rows = json.load(open('rows.json'))
fileTotal = collections.Counter(r['file'] for r in rows)
inst = collections.defaultdict(set); mut = collections.Counter()
for r in rows:
    if r['fn']:
        inst[(r['file'], r['fn'])].add(r['fnstart']); mut[(r['file'], r['fn'])] += 1
TOTAL = len(rows)


def git(*a):
    return subprocess.run(['git', *a], capture_output=True, cwd=R).stdout.decode('utf8', errors='replace')


def test_spans(s, toks):
    out = []
    n = len(toks)
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


def analyse(s):
    toks = tokens(s)
    fs = fn_spans(s)
    ts = test_spans(s, toks)
    out = []
    for f in fs:
        body = s[f['hdr']:f['end']]
        ftoks = [(k, x) for k, x, a, b in toks if f['hdr'] <= a < f['end']]
        tokstream = ' '.join((x if k != 'str' else 'S') for k, x in ftoks)
        strs = [s[a:b] for k, x, a, b in toks if k == 'str' and f['hdr'] <= a < f['end']]
        is_test = any(a <= f['start'] < b for a, b in ts) or bool(re.search(r'#\s*\[\s*test\s*\]', s[f['hdr']:f['start']]))
        line = s.count('\n', 0, f['start']) + 1
        out.append(dict(name=f['name'], s=hashlib.sha1(re.sub(r'\s+', '', body).encode()).hexdigest(),
                        t=hashlib.sha1((tokstream + '\0' + '\0'.join(strs)).encode()).hexdigest(), test=is_test, line=line))
    outside = ' '.join(s[a:b] for k, x, a, b in toks if not any(f['hdr'] <= a < f['end'] for f in fs))
    return out, outside


def mutants_of(f, name, k):
    key = (f, name)
    if key in inst:
        return mut[key] * min(k, len(inst[key])) / len(inst[key])
    return 0


shas = git('log', '--no-merges', '-150', '--format=%h', '--', 'crates').split()
res = []
for c in shas:
    allch = git('diff', '--name-only', c + '^', c).split()
    ch = [f for f in allch if f.startswith('crates')]
    lf = [f for f in ch if f in fileTotal]
    if not lf: continue
    if c == '53c9f384': continue          # 整体导入，没有旧版可比
    eff = {'s': 0.0, 't': 0.0}; fn_only = {'s': 0.0, 't': 0.0}; out_files = 0
    changed_fn = {'s': 0, 't': 0}; changed_test_fn = 0; ws_only_fn = 0; ws_only_mut = 0.0
    shifted_mut = 0.0; carried_in_changed_files = 0.0
    for f in lf:
        new = git('show', f'{c}:{f}'); old = git('show', f'{c}^:{f}')
        if not old:
            for m in eff: eff[m] += fileTotal[f]; fn_only[m] += fileTotal[f]
            continue
        (fo, oo), (fnw, on) = analyse(old), analyse(new)
        outside_changed = (oo != on)
        out_files += outside_changed
        for m in ('s', 't'):
            so = collections.Counter((x['name'], x[m]) for x in fo)
            cnt = collections.Counter((x['name'], x[m]) for x in fnw)
            diff = cnt - so
            chg = collections.Counter()
            for (name, h), k in diff.items(): chg[name] += k
            mm = sum(mutants_of(f, name, k) for name, k in chg.items())
            changed_fn[m] += sum(chg.values())
            fn_only[m] += mm
            eff[m] += fileTotal[f] if outside_changed else mm
        sold = collections.Counter((x['name'], x['s']) for x in fo)
        for x in fnw:
            if sold[(x['name'], x['s'])] > 0:
                sold[(x['name'], x['s'])] -= 1
            elif x['test']:
                changed_test_fn += 1
        sd = collections.Counter((x['name'], x['s']) for x in fnw) - collections.Counter((x['name'], x['s']) for x in fo)
        td = collections.Counter((x['name'], x['t']) for x in fnw) - collections.Counter((x['name'], x['t']) for x in fo)
        sdn = collections.Counter(); tdn = collections.Counter()
        for (nm, h), k in sd.items(): sdn[nm] += k
        for (nm, h), k in td.items(): tdn[nm] += k
        for nm in sdn:
            extra = sdn[nm] - tdn.get(nm, 0)
            if extra > 0:
                ws_only_fn += extra
                ws_only_mut += mutants_of(f, nm, extra)
        old_by = collections.defaultdict(list)
        for x in fo: old_by[(x['name'], x['t'])].append(x['line'])
        used = collections.Counter()
        for x in fnw:
            key = (x['name'], x['t'])
            if key in old_by and used[key] < len(old_by[key]):
                ol = old_by[key][used[key]]; used[key] += 1
                n_m = mutants_of(f, x['name'], 1)
                carried_in_changed_files += n_m
                if ol != x['line']: shifted_mut += n_m
    cats = dict(
        test_files=sum(1 for p in allch if re.match(r'crates/[^/]+/tests/', p)),
        fmt=any(p.startswith('crates/singlefs-format/') for p in allch),
        harness_src=any(p.startswith('crates/singlefs-harness/src/') for p in allch),
        tier_src=any(p.startswith('crates/singlefs-checker-tier/src/') for p in allch),
        cfg=any(p in ('Cargo.toml', 'Cargo.lock') or p.startswith('.cargo/') or re.match(r'crates/[^/]+/Cargo.toml', p) for p in allch),
        data=any(p.startswith('litmus/') or p == '.claude/gate.d/stage-inputs.tsv' for p in allch),
    )
    res.append(dict(c=c, files=len(lf), eff_s=round(eff['s']), eff_t=round(eff['t']), fn_s=round(fn_only['s']), fn_t=round(fn_only['t']),
                    out_files=out_files, changed_fn_s=changed_fn['s'], changed_fn_t=changed_fn['t'], changed_test_fn=changed_test_fn,
                    ws_only_fn=ws_only_fn, ws_only_mut=round(ws_only_mut), shifted=round(shifted_mut), carried_in_changed=round(carried_in_changed_files), **cats))
json.dump(res, open('c1_commits.json', 'w'), ensure_ascii=False)
print('n commits', len(res), 'list size', TOTAL)
print('commit eff_s eff_t fn_s fn_t outFiles chgFn_s chgFn_t chgTestFn wsOnlyFn wsOnlyMut | testFiles fmt harnessSrc tierSrc cfg data | carried-in-changed-files shifted')
for r in res:
    print(r['c'], r['eff_s'], r['eff_t'], r['fn_s'], r['fn_t'], r['out_files'], r['changed_fn_s'], r['changed_fn_t'], r['changed_test_fn'], r['ws_only_fn'], r['ws_only_mut'], '|',
          r['test_files'], int(r['fmt']), int(r['harness_src']), int(r['tier_src']), int(r['cfg']), int(r['data']), '|', r['carried_in_changed'], r['shifted'])
for k in ('eff_s', 'eff_t', 'fn_s', 'changed_fn_s', 'changed_test_fn'):
    v = sorted(r[k] for r in res)
    print('median', k, statistics.median(v), 'p25', v[len(v) // 4], 'p75', v[3 * len(v) // 4], 'min', v[0], 'max', v[-1])
print('commits with >=1 changed test fn in listed files:', sum(1 for r in res if r['changed_test_fn'] > 0), 'of', len(res))
print('commits with changed tests/ files:', sum(1 for r in res if r['test_files'] > 0), 'of', len(res))
print('commits with format crate changed:', sum(1 for r in res if r['fmt']), 'of', len(res))
print('commits with harness src changed:', sum(1 for r in res if r['harness_src']), 'of', len(res))
print('commits with cfg changed:', sum(1 for r in res if r['cfg']), 'of', len(res))
print('commits with data changed:', sum(1 for r in res if r['data']), 'of', len(res))
anyflag = lambda r: r['changed_test_fn'] > 0 or r['test_files'] > 0 or r['fmt'] or r['harness_src'] or r['cfg'] or r['data']
print('commits with ANY of (test fn / tests dir / format / harness src / cfg / data):', sum(1 for r in res if anyflag(r)), 'of', len(res))
tot_c = sum(r['carried_in_changed'] for r in res); tot_s = sum(r['shifted'] for r in res)
print('carried-in-changed-files total', tot_c, 'shifted-start-line', tot_s, '%.1f%%' % (100 * tot_s / max(1, tot_c)))
