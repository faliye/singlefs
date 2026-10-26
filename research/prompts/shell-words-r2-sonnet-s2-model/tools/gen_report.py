#!/usr/bin/env python3
"""gen_report.py <登记结果目录> <输出目录>：把 out/*.jsonl 排成报告用的几张表。
写 <输出目录>/legend.md（判据句对照表：名字 → 引文 → 文件:行号）、table.md（逐条表，同命令同应当同实际的合并成一行）、stats.txt。"""
import sys; sys.dont_write_bytecode = True
import collections, glob, json, os, re, subprocess, sys
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from frag import F
REPO = '/home/fy5090/code/singlefs/'
PATHS = {'H': '.claude/hooks/heavy-test-guard.sh', 'B': '.claude/hooks/bash-command-detector.sh', 'L': '.claude/hooks/lib_heavy_tests.py',
         'W': '.claude/hooks/lib_shell_words.py', 'MA': '.claude/main-agent.md', 'AC': '.claude/agent-common.md'}
src, out = sys.argv[1], sys.argv[2]
os.makedirs(out, exist_ok=True)
rows = []
for f in sorted(glob.glob(src + '/*.jsonl')):
    for l in open(f): rows.append(json.loads(l))
def line_of(key):
    f, frag = F[key]
    r = subprocess.run(['grep', '-nF', '--', frag, REPO + PATHS[f]], capture_output=True, text=True)
    hits = [int(x.split(':', 1)[0]) for x in r.stdout.splitlines()]
    return PATHS[f], (hits[0] if hits else None), len(hits)
def shown(frag):
    return re.sub(r'「[^」]*」', '…', frag)
used = []
for r in rows:
    for c in r['cites']:
        if c not in used: used.append(c)
legend = ['| 名 | 判据句（原文）与位置 |', '|---|---|']
bad_frag = []
for c in used:
    path, ln, n = line_of(c)
    if ln is None: bad_frag.append(c); continue
    legend.append(f"| {c} | 「{shown(F[c][1])}」（`{path}:{ln}`） |")
open(out + '/legend.md', 'w').write('\n'.join(legend) + '\n')
SHORT = {'main': '主', 'implementation-writer': '实现员', 'experiment-runner': '执行员', 'three-way-forward': '正推腿', 'clerk': '书记员', 'investigator': '调查员',
         'tooling-writer': '工具实现员', 'general-purpose': '通用', 'three-way-attack': '攻方腿', 'mutation-triage': '变异分诊', 'kb-scribe': '写回员', 'sweep': 'sweep',
         'prior-art': '查前人', 'three-way-materials': '材料员', 'three-way-defense': '辩方腿', 'experiment-designer': '设计员'}
def esc(s, n=110):
    s = re.sub(r'(\.[A-Za-z]+):(\d+)', r'\1 第\2 行', s)
    s = s.replace('\\', '\\\\').replace('\n', '⏎').replace('|', '\\|').replace('`', "'")
    return s if len(s) <= n else s[:n] + '…'
def verdict(r):
    if r['unspecified']: return '规则没说'
    ok_h, ok_d = r['rc_h'] == r['exp_h'], r['rc_d'] == r['exp_d']
    if ok_h and ok_d: return '对'
    wrong = []
    if not ok_h: wrong.append('误拒' if r['exp_h'] == 0 else '漏拒')
    if not ok_d: wrong.append('误拒' if r['exp_d'] == 0 else '漏拒')
    return '错：' + '+'.join(wrong)
groups = collections.OrderedDict()
for r in rows:
    k = (r['cmd'], r['cwd'], r['bg'], r['exp_h'], r['exp_d'], r['rc_h'], r['rc_d'], verdict(r), r['unspecified'])
    groups.setdefault(k, []).append(r)
head = ['| 编号 | 命令（⏎ 是换行） | 喂 | 身份 | 判据句 | 应当 H/D | 实际 H/D（拒时标准错误首行） | 判 |', '|---|---|---|---|---|---|---|---|']
lines = list(head)
stats = collections.Counter()
for k, g in groups.items():
    cmd, cwd, bg, eh, ed, rh, rd, v, uns = k
    ids = [x['id'] for x in g]
    idtxt = ids[0] if len(ids) == 1 else f"{ids[0]} 等{len(ids)}条"
    idents = '、'.join(dict.fromkeys(SHORT.get(x['ident'], x['ident']) for x in g))
    fed = '两份' if eh == 0 and ed == 0 else ('H' if ed == 0 else ('D' if eh == 0 else 'H+D'))
    opt = (f"cwd={cwd or '默认'}" if cwd else '') + (' 后台' if bg else '')
    first = g[0]
    err = ''
    if rh: err += ' H:' + esc(first['err_h'].replace('✗ ', ''), 60)
    if rd: err += ' D:' + esc(first['err_d'].replace('✗ ', ''), 60)
    cites = ' '.join(first['cites'])
    lines.append(f"| {idtxt} | `{esc(cmd)}`{(' ['+opt+']') if opt else ''} | {fed} | {idents} | {cites} | {eh}/{ed} | {rh}/{rd}{err} | {v} |")
    stats[v] += len(g)
open(out + '/table.md', 'w').write('\n'.join(lines) + '\n')
with open(out + '/stats.txt', 'w') as f:
    f.write(f"rows {len(rows)}\n")
    for k, v in stats.items(): f.write(f"{k}\t{v}\n")
    f.write(f"groups {len(groups)}\nbad_fragments {bad_frag}\n")
print(open(out + '/stats.txt').read())
