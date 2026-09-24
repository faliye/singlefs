#!/usr/bin/env python3
"""T1 只读模型：在真仓 git 历史上，对 10 号第 3 段的对象（工作区里已跑、被 decisions/ 正文引用的实验）
逐个算三种取法挑中的提交与它动没动决策：
  甲  git log -1 -S"## E<n> "（10 号现状，第 119 行）
  乙  -G 找候选，取最新一个「该实验的 + 标题行带『已跑』而 − 标题行不带」的提交（正文第 41 行的写法）
  终  最新一个「该实验标题行的状态段变了、且 + 行带『已跑』」的提交（含 部分已跑 → 已跑）
不写仓，只读 git。用法：python3 t1-history.py <仓根>
"""
import glob, os, re, subprocess, sys

root = sys.argv[1]
os.chdir(root)
KB = '.claude/kb'
PATHS = [f'{KB}/experiments', f'{KB}/experiments.md']

def git(*args):
    return subprocess.run(['git', '-c', 'core.quotePath=false', *args], capture_output=True, text=True).stdout

exp_files = sorted(glob.glob(f'{KB}/experiments/*.md'))
dec_files = sorted(glob.glob(f'{KB}/decisions/*.md'))
ran = []
for f in exp_files:
    for line in open(f, encoding='utf-8'):
        m = re.match(r'^## (E[0-9]+) ', line)
        if m and '已跑' in line[3:]:
            ran.append(m.group(1))
dec_text = {f: open(f, encoding='utf-8').read() for f in dec_files}

def referenced(e):
    pat = re.compile(r'(^|[^A-Za-z0-9/-])' + e + r'([^A-Za-z0-9-]|$)', re.M)
    return [f for f, t in dec_text.items() if pat.search(t)]

def touched_decisions(c):
    names = git('show', '--name-only', '--format=', c).split('\n')
    return any(re.match(r'^\.claude/kb/decisions(\.md$|/)', n) for n in names)

def status(line):
    # 标题行「—— 」之后的状态段；没有破折号就取整行
    return line.split('——', 1)[1].strip() if '——' in line else line

rows = []
for e in ran:
    refs = referenced(e)
    if not refs:
        continue
    jia = git('log', '-1', '--format=%h', f'-S## {e} ', '--', *PATHS).strip()
    cands = git('log', '--format=%h', f'-G^## {e} ', '--', *PATHS).split()
    yi = zhong = ''
    for c in cands:  # 新 → 旧
        diff = git('show', '-U0', '--format=', c, '--', *PATHS).split('\n')
        plus = [l[1:] for l in diff if l.startswith(f'+## {e} ')]
        minus = [l[1:] for l in diff if l.startswith(f'-## {e} ')]
        if not yi and any('已跑' in p for p in plus) and not any('已跑' in m for m in minus):
            yi = c
        if not zhong and any('已跑' in p for p in plus) and \
                {status(p) for p in plus} != {status(m) for m in minus} and \
                not ({re.sub(r'（.*', '', status(p)) for p in plus} <= {re.sub(r'（.*', '', status(m)) for m in minus}):
            zhong = c
        if yi and zhong:
            break
    rows.append((e, jia, jia and touched_decisions(jia), yi, yi and touched_decisions(yi), zhong, zhong and touched_decisions(zhong)))

print('| 实验 | 甲 提交 | 甲 动决策 | 乙 提交 | 乙 动决策 | 终 提交 | 终 动决策 |')
print('|---|---|---|---|---|---|---|')
for r in rows:
    print('| ' + ' | '.join(str(x) if x != '' else '—' for x in r) + ' |')
n = len(rows)
jr = sum(1 for r in rows if r[1] and not r[2])
yr = sum(1 for r in rows if r[3] and not r[4])
yn = sum(1 for r in rows if not r[3])
zr = sum(1 for r in rows if r[5] and not r[6])
diff_yz = [r[0] for r in rows if r[3] != r[5]]
print(f'对象 {n} 个；甲 红 {jr}；乙 红 {yr}、乙 找不到提交 {yn}；终 红 {zr}')
print(f'乙 与 终 挑的提交不同的实验 {len(diff_yz)} 个：{" ".join(diff_yz)}')
print('乙 与 终 挑的提交不同、且判定不同的：' + ' '.join(
    f'{r[0]}(乙 {r[3]}:{"绿" if r[4] else "红"} / 终 {r[5]}:{"绿" if r[6] else "红"})'
    for r in rows if r[3] != r[5] and bool(r[4]) != bool(r[6])))
