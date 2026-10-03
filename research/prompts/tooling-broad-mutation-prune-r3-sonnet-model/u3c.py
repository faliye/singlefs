"""U3 最后一张表：每条变异平均秒数的四个读法，各自标「量的」还是「推的」；代进 14 个提交的 C1 条数与全量 5091 条。
输入全部取自库里实测（results.tsv 的 302 条、baseline.tsv、baseline.one-test.log），只读；不起 cargo。"""
import csv, os, re, json, statistics, collections
V = os.path.expanduser('~/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4')
dur = {}
for l in open(V + '/logs/baseline.one-test.log', errors='replace'):
    m = re.match(r'\s*PASS \[\s*([\d.]+)s\]\s+\(\s*\d+/\d+\)\s+(.*?)\s*$', l)
    if m: dur[m.group(2)] = float(m.group(1))
last = {}
for r in csv.reader(open(V + '/results.tsv'), delimiter='\t'):
    if r and not r[0].startswith('#'): last[int(r[0])] = r
cat = collections.defaultdict(list)
for r in last.values(): cat[r[3]].append(r)
n = len(last)
sh = {k: len(v) / n for k, v in cat.items()}
mb = {k: statistics.mean(float(r[6]) for r in v) for k, v in cat.items()}
mt = {k: statistics.mean(float(r[7]) for r in v) for k, v in cat.items()}
tfirst = statistics.mean(float(r[6]) + dur[r[5]] for r in cat['caught'])
base = {l.split('\t')[0]: float(l.split('\t')[2]) for l in open(V + '/baseline.tsv') if not l.startswith('#')}
stage_two = base['two-test-doc'] + base['two-test-slow-harness'] + base['two-test-checker-tier'] + base['two-build-harness'] + base['two-build-checker-tier']
def reading_i():   # 今天：第一段，量的
    return sum(sh[k] * (mb[k] + mt[k]) for k in cat)
def reading_ii(L):  # C2 的 T-first（抓到的）+ C4 把超时压到 L 秒；幸存条不省；不含第二段
    return sh['caught'] * tfirst + sh['unviable'] * mb['unviable'] + sh['timeout'] * (mb['timeout'] + L) + sh['stage-one-only'] * (mb['stage-one-only'] + mt['stage-one-only'])
def reading_iii():  # 只算编译：每条一次增量编译，测试不花时间
    return sum(sh[k] * mb[k] for k in cat)
def addon_stage_two():  # 幸存条的第二段，按基线秒数当上界
    return sh['stage-one-only'] * stage_two
print('stage two baseline seconds per survivor (tests + builds): %.1f' % stage_two)
print('per-mutant seconds: (i) today first stage, measured %.1f | (ii) C2 T-first + C4 cap L=60 %.1f, L=300 %.1f | (iii) compile only %.1f | stage-two add-on for survivors (upper bound) %.1f' % (
    reading_i(), reading_ii(60), reading_ii(300), reading_iii(), addon_stage_two()))
res = json.load(open('c1_commits.json'))
effs = sorted(r['eff_s'] for r in res)
med = statistics.median(effs)
def H(c, s): return c * s / 3600
print('median C1 count over 14 commits: %d' % med)
print('median commit hours: (i) %.1f | (ii) L=60 %.1f L=300 %.1f | (iii) %.1f | stage-two add-on %.1f' % (H(med, reading_i()), H(med, reading_ii(60)), H(med, reading_ii(300)), H(med, reading_iii()), H(med, addon_stage_two())))
print('all 5091 hours: (i) %.1f | (ii) L=60 %.1f L=300 %.1f | (iii) %.1f | stage-two add-on %.1f' % (H(5091, reading_i()), H(5091, reading_ii(60)), H(5091, reading_ii(300)), H(5091, reading_iii()), H(5091, addon_stage_two())))
print('per commit: commit C1-count | (i) | (ii) L=300 | (iii)')
for r in sorted(res, key=lambda x: x['eff_s']):
    e = r['eff_s']
    print('%s %5d | %6.1f | %6.1f | %5.1f' % (r['c'], e, H(e, reading_i()), H(e, reading_ii(300)), H(e, reading_iii())))
print('commits with C1 count <= 200: %d of 14 ; hours (i) at 200: %.1f' % (sum(1 for e in effs if e <= 200), H(200, reading_i())))
print('commits with C1 count >= 3000: %d of 14' % sum(1 for e in effs if e >= 3000))
