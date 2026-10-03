"""U3 的算术：每条变异的单价按结局分类取（库里 results.tsv 的实测），代进 C1+C2+C4 的三种读法；14 个真实提交与里程碑末全量。
量的：results.tsv 每行的 build_seconds / test_seconds、baseline.tsv、baseline.one-test.log 里每条测试的耗时。
推的：C2 挑出来的测试集有多大（没有覆盖产物）、C4 的限时 L、幸存条的第二段、插桩倍数。这些在下面全写成参数，不写成结论。
只读库目录与 c1_commits.json；不起 cargo。"""
import re, json, csv, collections, statistics, os
V = os.path.expanduser('~/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4')
dur = {}
for l in open(V + '/logs/baseline.one-test.log', errors='replace'):
    m = re.match(r'\s*PASS \[\s*([\d.]+)s\]\s+\(\s*\d+/\d+\)\s+(.*?)\s*$', l)
    if m: dur[m.group(2)] = float(m.group(1))
rows = [r for r in csv.reader(open(V + '/results.tsv'), delimiter='\t') if r and not r[0].startswith('#')]
last = {}
for r in rows: last[int(r[0])] = r
n = len(last)
cat = collections.defaultdict(list)
for s, r in last.items():
    cat[r[3]].append((float(r[6]), float(r[7]), r[5]))
print('rows (last per sequence)', n, {k: len(v) for k, v in cat.items()})
def mean(x): return sum(x) / len(x) if x else 0.0
B = {k: mean([b for b, t, T in v]) for k, v in cat.items()}
Tm = {k: mean([t for b, t, T in v]) for k, v in cat.items()}
for k in cat: print('  %-15s n=%3d build mean %.1f | test mean %.1f' % (k, len(cat[k]), B[k], Tm[k]))
tonly = [b + dur[T] for b, t, T in cat['caught'] if T in dur]
print('caught with T in baseline log:', len(tonly), 'of', len(cat['caught']), '| build + baseline duration of T: mean %.1f median %.1f' % (mean(tonly), statistics.median(tonly)))
share = {k: len(v) / n for k, v in cat.items()}
print('shares', {k: round(x, 4) for k, x in share.items()})
# 第二段（只对「第一段没红」的）：基线里各步的秒数（baseline.tsv，量的）。每条幸存条的第二段上界 = 这几步基线秒数相加（不含编译）
base = {l.split('\t')[0]: float(l.split('\t')[2]) for l in open(V + '/baseline.tsv') if not l.startswith('#')}
print('baseline.tsv steps', base)
stage_two_test = base['two-test-doc'] + base['two-test-slow-harness'] + base['two-test-checker-tier']
stage_two_build = base['two-build-harness'] + base['two-build-checker-tier']
print('stage two per survivor, baseline seconds: tests %.1f + builds %.1f (cold-ish baseline builds, an upper bound for incremental)' % (stage_two_test, stage_two_build))
S = share['stage-one-only']
# 三种读法（每条变异平均秒数）
def per_mutant(reading, L):
    v = share['caught'] * {'today': B['caught'] + Tm['caught'], 'floor': B['caught'], 'tfirst': mean(tonly)}[reading]
    v += share['unviable'] * B['unviable']
    # 超时条：今天是 build + 整段总时限；C4 之后是 build + L（L 为单条测试的限时，参数）
    v += share['timeout'] * (B['timeout'] + (Tm['timeout'] if reading == 'today' else (0 if reading == 'floor' else L)))
    # 幸存条：今天是 build + 全部测试 + 第二段；C2 之后 floor = 只编、tfirst/upper 见下
    if reading == 'today': v += S * (B['stage-one-only'] + Tm['stage-one-only'] + stage_two_test + stage_two_build)
    elif reading == 'floor': v += S * B['stage-one-only']
    else: v += S * (B['stage-one-only'] + Tm['stage-one-only'] + stage_two_test)   # tfirst 读法对幸存条仍给上界：C2 对它们能省多少没有数
    return v
for L in (60, 300):
    print('L=%d s: per-mutant seconds today(measured, plus stage two inferred) %.1f | C2 floor(only compile) %.1f | C2 T-first(caught) %.1f' % (
        L, per_mutant('today', L), per_mutant('floor', L), per_mutant('tfirst', L)))
print('  stage-one-only share %.3f -> its part in tfirst: %.1f s of %.1f' % (S, S * (B['stage-one-only'] + Tm['stage-one-only'] + stage_two_test), per_mutant('tfirst', 300)))
res = json.load(open('c1_commits.json'))
fixed_low = (1426.0 / 3600)           # 基线七步（量的），未插桩
def hrs(count, reading, L): return count * per_mutant(reading, L) / 3600
print('--- 14 commits, C1 eff (s reading) -> hours; fixed per commit not included')
print('commit eff | today | tfirst L=60 | tfirst L=300 | floor')
tot = {'today': [], 'tf60': [], 'tf300': [], 'floor': []}
for r in sorted(res, key=lambda x: x['eff_s']):
    e = r['eff_s']
    a, b, c, d = hrs(e, 'today', 300), hrs(e, 'tfirst', 60), hrs(e, 'tfirst', 300), hrs(e, 'floor', 300)
    tot['today'].append(a); tot['tf60'].append(b); tot['tf300'].append(c); tot['floor'].append(d)
    print('%s %5d | %6.1f | %6.1f | %6.1f | %6.1f' % (r['c'], e, a, b, c, d))
for k, v in tot.items(): print('median', k, '%.1f h' % statistics.median(v), 'min %.1f max %.1f' % (min(v), max(v)))
N = 5091
print('--- milestone-end full list (5091), no coverage pass, hours: today %.1f | tfirst L=60 %.1f | tfirst L=300 %.1f | floor %.1f' % (
    hrs(N, 'today', 300), hrs(N, 'tfirst', 60), hrs(N, 'tfirst', 300), hrs(N, 'floor', 300)))
judged = len(last); print('judged today', judged, 'unjudged', N - judged)
print('compile-only floor if every viable mutant compiles once: viable share %.3f x %.1f s x 5091 = %.1f h' % (
    share['caught'] + share['timeout'] + share['stage-one-only'],
    mean([b for k in ('caught', 'timeout', 'stage-one-only') for b, t, T in cat[k]]),
    (share['caught'] + share['timeout'] + share['stage-one-only']) * mean([b for k in ('caught', 'timeout', 'stage-one-only') for b, t, T in cat[k]]) * N / 3600))
# 固定量：基线七步 + 覆盖遍（倍数是参数）
for m in (1.0, 1.5, 2.0, 3.0):
    print('fixed per version: baseline %.2f h + coverage pass at multiplier %.1f = %.2f h (coverage multiplier 没量)' % (fixed_low, m, fixed_low * (1 + m)))
# 覆盖遍后处理：一条测试一个 profraw、一次 export；个数是量的，每个的秒数是参数
n_tests_stage1 = len(dur)
print('per-test processes in coverage pass: stage-one tests (from baseline log) %d ; slow ignored 61 (baseline.two-test-slow-harness.log header); checker-tier libtest tests: 436 #[test] fns - not countable per-process without running' % n_tests_stage1)
for sec in (1, 3, 10):
    print('  post-processing at %d s per test process, ~1273 processes: %.2f h' % (sec, 1273 * sec / 3600))
