#!/usr/bin/env python3
"""T5：--no-renames 让搬家之后整页算新增。真仓里哪几页一搬家，88 / 40 号就要判到早先写下、今天对不上的行。

只读：在第一个参数给的仓根上跑（要它的 git 历史），取法照快照树里的 88、40 号逐行抄：
  88：kb 正文（历史版本之前）与 research/**/*.md（prompts、results 除外）里去掉首尾空白以 `E7RESULT ` 开头的行，
      在 research/results/*.out、git log --all --diff-filter=D 删掉的 .out、HEAD 里有而工作区没有的 .out 里逐字找。
  40：experiments.md 与 experiments/*.md 里点名的 e<数字>…​.out，树里、HEAD 里、git log --all --diff-filter=D 里找。
用法：python3 t5-moved-pages.py <仓根>
"""
import glob, os, re, subprocess, sys

root = sys.argv[1]
os.chdir(root)

def git(*args):
    return subprocess.run(['git', '-c', 'core.quotepath=false', *args], capture_output=True, text=True)

# ── 88 ──
kb = sorted(f for f in glob.glob('.claude/kb/**/*.md', recursive=True) + glob.glob('research/**/*.md', recursive=True)
            if not f.endswith('-history.md') and '/decisions-history/' not in f
            and not f.startswith('research/prompts/') and not f.startswith('research/results/'))
quoted = []
for path in kb:
    text = open(path, encoding='utf-8').read()
    cut = text.find('\n## 历史版本')
    body = text if cut < 0 else text[:cut]
    for number, line in enumerate(body.split('\n'), 1):
        if line.strip().startswith('E7RESULT '):
            quoted.append((path, number, line.strip()))
product_lines = set()
for product in sorted(glob.glob('research/results/*.out')):
    with open(product, encoding='utf-8', errors='ignore') as handle:
        for line in handle:
            product_lines.add(line.strip().replace('\r', ''))
archived = set()
log = git('log', '--all', '--diff-filter=D', '--format=%H', '--name-only', '--', 'research/results').stdout
commit = None
for entry in log.split('\n'):
    entry = entry.strip()
    if not entry:
        continue
    if len(entry) == 40 and all(c in '0123456789abcdef' for c in entry):
        commit = entry; continue
    if commit and entry.endswith('.out'):
        blob = git('show', '%s^:%s' % (commit, entry))
        if blob.returncode == 0:
            archived |= {one.strip().replace('\r', '') for one in blob.stdout.split('\n')}
for entry in git('ls-tree', '-r', '--name-only', 'HEAD', '--', 'research/results').stdout.split('\n'):
    if entry.endswith('.out') and not os.path.exists(entry):
        blob = git('show', 'HEAD:%s' % entry)
        if blob.returncode == 0:
            archived |= {one.strip().replace('\r', '') for one in blob.stdout.split('\n')}
missing88 = [q for q in quoted if q[2] not in product_lines and q[2] not in archived]
pages88 = sorted({p for p, _, _ in missing88})
print(f'88：{len(kb)} 份正文里整行抄的 E7RESULT 行 {len(quoted)} 行；树里、归档、HEAD 删掉的产物里都找不到的 {len(missing88)} 行，分在 {len(pages88)} 份：')
for page in pages88:
    rows = [q for q in missing88 if q[0] == page]
    print(f'  {page}：{len(rows)} 行，例 第 {rows[0][1]} 行 {rows[0][2][:100]}')

# ── 40 ──
EXP = ['.claude/kb/experiments.md'] + sorted(glob.glob('.claude/kb/experiments/*.md'))
named = {}
for path in EXP:
    for number, line in enumerate(open(path, encoding='utf-8').read().split('\n'), 1):
        for match in re.finditer(r'(^|[^a-zA-Z0-9_-])(e[0-9]+[a-zA-Z0-9._-]*\.out)', line):
            named.setdefault(match.group(2), []).append((path, number))
missing40 = []
for name, where in sorted(named.items()):
    if os.path.isfile('research/results/' + name):
        continue
    if git('cat-file', '-e', 'HEAD:research/results/' + name).returncode == 0:
        continue
    if git('log', '--all', '--diff-filter=D', '--format=%h', '--name-only', '--', '*' + name).stdout.strip():
        continue
    missing40.append((name, where))
pages40 = sorted({p for _, where in missing40 for p, _ in where})
print(f'40：实验正文点名的产物 {len(named)} 份；树里、HEAD、git 历史里都没有的 {len(missing40)} 份，分在 {len(pages40)} 页：')
for name, where in missing40:
    print(f'  {name}：{", ".join(f"{p}:{n}" for p, n in where[:3])}')
