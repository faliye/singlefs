#!/usr/bin/env python3
"""T1 模型的判定器（只读 git 与工作区）：对一个实验号 E，按四种取法判 10 号第 3 段那一格。
  乙      -G 候选里最新一个「+ 标题行带『已跑』、− 标题行不带」的提交，动没动决策（正文第 41 行）
  丙a     基准（changed-paths.sh 的 gate 取法）那一版标题不带『已跑』、工作区带：改动范围里有没有任一份决策
  丙b     同上，但要求引用它的那份决策正文在改动范围里
  丙a*/丙b* 同丙a/丙b，但「状态变成已跑」按状态段逐字判（部分已跑 → 已跑 也算）
甲由真仓 10 号自己判（调用方直接跑它）。用法：python3 t1-judge.py <E号> <changed-paths.sh 路径>
"""
import glob, os, re, subprocess, sys
e, lib = sys.argv[1], sys.argv[2]
P = ['.claude/kb/experiments', '.claude/kb/experiments.md']
def git(*a):
    return subprocess.run(['git', '-c', 'core.quotePath=false', *a], capture_output=True, text=True).stdout
def touched_dec(names):
    return any(re.match(r'^\.claude/kb/decisions(\.md$|/)', n) for n in names)
def heading_now():
    for f in glob.glob('.claude/kb/experiments/*.md') + glob.glob('.claude/kb/experiments.md'):
        for l in open(f, encoding='utf-8'):
            if l.startswith(f'## {e} '):
                return l.rstrip('\n')
    return ''
def heading_at(base):
    names = git('ls-tree', '-r', '--name-only', base, '--', *P).split('\n')
    for n in names:
        if not n:
            continue
        for l in git('show', f'{base}:{n}').split('\n'):
            if l.startswith(f'## {e} '):
                return l
    return ''
def status(l):
    return l.split('——', 1)[1].strip() if '——' in l else ''
def ran_token(l):  # 状态段去掉括注后恰好是「已跑」
    return re.sub(r'（.*', '', status(l)).strip() == '已跑'
refs = [f for f in glob.glob('.claude/kb/decisions/*.md')
        if re.search(r'(^|[^A-Za-z0-9/-])' + e + r'([^A-Za-z0-9-]|$)', open(f, encoding='utf-8').read(), re.M)]
# 乙
yi = ''
for c in git('log', '--format=%h', f'-G^## {e} ', '--', *P).split():
    d = git('show', '-U0', '--format=', c, '--', *P).split('\n')
    plus = [l for l in d if l.startswith(f'+## {e} ')]; minus = [l for l in d if l.startswith(f'-## {e} ')]
    if any('已跑' in l for l in plus) and not any('已跑' in l for l in minus):
        yi = c; break
if not yi:
    print('乙: 没判（历史里没有变成已跑的提交）')
else:
    print(f'乙: 提交 {yi} ' + ('动了决策 → 绿' if touched_dec(git('show', '--name-only', '--format=', yi).split('\n')) else '没动决策 → 红'))
# 丙
base = subprocess.run(['bash', '-c', f'source "{lib}"; gate_diff_base gate'], capture_output=True, text=True).stdout.strip()
changed = subprocess.run(['bash', '-c', f'source "{lib}"; gate_changed_paths "{base}" untracked'], capture_output=True, text=True).stdout.split('\n')
before, now = heading_at(base), heading_now()
for name, became in (('丙a/丙b', ('已跑' not in before) and ('已跑' in now)),
                     ('丙a*/丙b*', (not ran_token(before)) and ran_token(now) and status(before) != status(now))):
    if not became:
        print(f'{name}: 这一批里它没有变成已跑 → 不判')
        continue
    a = touched_dec(changed); b = any(r in changed for r in refs)
    print(f'{name}: 基准 {git("log", "-1", "--format=%s", base).strip()}；任一决策在范围里 {a} → {"绿" if a else "红"}；引用它的决策在范围里 {b} → {"绿" if b else "红"}')
