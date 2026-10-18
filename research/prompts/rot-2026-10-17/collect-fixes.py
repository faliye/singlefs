#!/usr/bin/env python3
"""把各段报告「## 逐行判定」表里判「要改」「要补」「要人看」的行抽出来，按载体合成 fixes.tsv（判定、组、段、载体、改后句）。
用法：python3 collect-fixes.py 报告目录 fixes.tsv
"""
import sys, glob, re, os, collections
reports, out = sys.argv[1], sys.argv[2]
rows = []
for path in sorted(glob.glob(os.path.join(reports, 'judge-S*.md'))):
    seg = re.search(r'judge-(S\d+)', path).group(1)
    text = open(path, encoding='utf-8', errors='replace').read()
    section = text.split('## 逐行判定', 1)[1] if '## 逐行判定' in text else ''
    section = re.split(r'\n## ', section, 1)[0]
    for line in section.splitlines():
        if not line.startswith('|'): continue
        cells = [c.strip() for c in line.strip().strip('|').split('|')]
        if len(cells) < 4 or cells[0] in ('组', '---') or cells[0].startswith('-'): continue
        verdict = cells[2]
        if verdict.startswith(('要改', '要补', '要人看')):
            rows.append((verdict.split('（')[0][:3], cells[0], seg, cells[1], '|'.join(cells[3:])))
by_carrier = collections.defaultdict(list)
for r in rows: by_carrier[r[3].rsplit(':', 1)[0]].append(r)
with open(out, 'w', encoding='utf-8') as f:
    f.write('判定\t组\t段\t载体\t改后句或理由\n')
    for carrier in sorted(by_carrier):
        for r in sorted(by_carrier[carrier], key=lambda x: int(x[3].rsplit(':', 1)[1]) if x[3].rsplit(':', 1)[1].isdigit() else 0):
            f.write('\t'.join(r) + '\n')
counts = collections.Counter(r[0] for r in rows)
print(f'  ✓ {out}：{len(rows)} 行（{dict(counts)}），{len(by_carrier)} 份载体；按载体前 15：')
for carrier, items in sorted(by_carrier.items(), key=lambda x: -len(x[1]))[:15]:
    print(f'     {len(items):4d}  {carrier}')
