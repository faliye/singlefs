"""U3 补一行：库里 302 条已判的实测总秒数（build_seconds + test_seconds，只含第一段）按结局拆开，看尾巴占多少；再按 C1 的条数乘出小时。只读库目录与 c1_commits.json。"""
import csv, os, json, statistics, collections
V = os.path.expanduser('~/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4')
last = {}
for r in csv.reader(open(V + '/results.tsv'), delimiter='\t'):
    if r and not r[0].startswith('#'): last[int(r[0])] = r
tot = collections.Counter(); cnt = collections.Counter()
for r in last.values():
    tot[r[3]] += float(r[6]) + float(r[7]); cnt[r[3]] += 1
S = sum(tot.values()); n = len(last)
print('rows', n, 'total measured seconds (first stage only) %.0f = %.1f h' % (S, S / 3600))
for k in tot: print('  %-15s n=%3d (%.1f%% of rows) seconds %6.0f (%.1f%% of seconds)' % (k, cnt[k], 100 * cnt[k] / n, tot[k], 100 * tot[k] / S))
per = S / n
print('measured mean per mutant, first stage only: %.1f s' % per)
res = json.load(open('c1_commits.json'))
v = sorted(r['eff_s'] for r in res)
print('median eff over 14 commits', statistics.median(v), '-> %.1f h at measured mean (first stage only, all outcomes)' % (statistics.median(v) * per / 3600))
print('all 5091 -> %.1f h at measured mean (first stage only, all outcomes)' % (5091 * per / 3600))
print('r2 steady state (caught+unviable only) 62.4 s -> median commit %.1f h ; all 5091 %.1f h' % (statistics.median(v) * 62.4 / 3600, 5091 * 62.4 / 3600))
