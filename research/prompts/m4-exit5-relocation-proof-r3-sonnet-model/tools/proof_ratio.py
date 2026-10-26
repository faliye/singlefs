#!/usr/bin/env python3
"""按现有 Verus 核（冻结副本里 singlefs-replay-prefix 的几份）现数「证明行对代码行」的比例，再对第二轮正推的 11 万–12.5 万行重算。
用法：proof_ratio.py <冻结副本> [<第二轮正推原型文件>…]
把每个 fn 单元按头一行分三类：spec fn / proof fn（含 open、closed、broadcast）/ 其余当 exec；文件里 fn 以外的行叫其他（类型、枚举、use、常量）。
行数只数去掉空行与 // 注释行之后的行。exec 的行里包括它的 requires / ensures / decreases 子句（分不开，所以 E 偏大、比例偏小，是下界）。
"""
import os, re, subprocess, sys, collections
root = sys.argv[1]
dump = subprocess.run(['python3', '-I', os.path.join(os.environ['REPO_ROOT'], 'scripts/gate/lib_function_units.py'), 'dump', root], capture_output=True, text=True).stdout
def code_lines(path):
    out = []
    for i, l in enumerate(open(path, encoding='utf-8'), 1):
        s = l.strip()
        out.append(bool(s) and not s.startswith('//'))
    return out
def kind_of(first):
    h = first.strip()
    if re.search(r'\bspec\s+fn\b', h) or re.search(r'\buninterp\s+spec\b', h): return 'spec'
    if re.search(r'\bproof\s+fn\b', h) or re.search(r'\baxiom\s+fn\b', h): return 'proof'
    return 'exec'
def measure(path):
    cl = code_lines(path)
    text = open(path, encoding='utf-8').read().split('\n')
    return cl, text
def verus_ranges(text):
    """verus! { … } 的行区间（括号配平，不看串与注释里的括号，对这几份核够用）。"""
    ranges = []; i = 0
    while i < len(text):
        if re.match(r'\s*verus!\s*\{', text[i]):
            depth = 0; j = i
            while j < len(text):
                depth += text[j].count('{') - text[j].count('}')
                if depth == 0 and j >= i: break
                j += 1
            ranges.append((i + 1, j + 1)); i = j + 1
        else:
            i += 1
    return ranges
def per_file(path, rows):
    cl, text = measure(path)
    vr = verus_ranges(text)
    inside = [any(a <= i + 1 <= b for a, b in vr) for i in range(len(cl))]
    cl = [c and inside[i] for i, c in enumerate(cl)]
    rows = [(a, b) for a, b in rows if any(x <= a and b <= y for x, y in vr)]
    covered = [False] * len(cl)
    sums = collections.Counter()
    for (a, b) in rows:
        k = kind_of(text[a - 1])
        for i in range(a - 1, b):
            if not covered[i]:
                covered[i] = True
                if cl[i]: sums[k] += 1
    sums['other'] = sum(1 for i, c in enumerate(cl) if c and not covered[i])
    return sums
files = collections.defaultdict(list)
for l in dump.splitlines():
    p = l.split('\t')
    if p[0] == 'fn' and 'singlefs-replay-prefix' in p[2]:
        f, span = p[2].rsplit(':', 1)
        a, b = (int(x) for x in span.split('-'))
        files[f].append((a, b))
tot = collections.Counter()
for f in sorted(files):
    s = per_file(f'{root}/{f}', files[f]); tot += s
    allc = sum(s.values())
    print(f'file={f.split("/")[-1]} code_lines={allc} exec={s["exec"]} spec={s["spec"]} proof={s["proof"]} other={s["other"]} ratio_all_over_exec={allc/max(s["exec"],1):.2f} ratio_nonexec_over_exec={(allc-s["exec"])/max(s["exec"],1):.2f}')
allc = sum(tot.values())
print(f'total code_lines={allc} exec={tot["exec"]} spec={tot["spec"]} proof={tot["proof"]} other={tot["other"]} ratio_all_over_exec={allc/max(tot["exec"],1):.2f}')
for path in sys.argv[2:]:
    pass
