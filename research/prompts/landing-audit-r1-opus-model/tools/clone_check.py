#!/usr/bin/env python3
"""攻方腿 landing-audit-r1 的整段相同核对（自己的模型，不是 gate-overlap.py）。
取法照 .claude/singlefs-ai-sop/scripts/gate-overlap.py 的 normalized_lines 与 CLONE_MINIMUM_LINES=8：
去空行、# 起头的行、去首尾空白并把行内连续空白压成一个之后短于 8 个字的行与只剩一个关键字的行。
与它不同的三处（都是有意的）：
  1. 对照范围加上 scripts/ 下全部 .sh / .py（gate-overlap 只比门禁、钩子与上游包的脚本）；
  2. 同一份文件里不重叠的两处也比（gate-overlap 只比别的文件）；
  3. 另开一档「引号归一」：把单引号换成双引号再比（gate-overlap 逐字比，'git' 与 "git" 算不同）。
用法：clone_check.py <仓根> <被判文件清单> ；清单一行一份「路径<制表符>new 或 diff」，diff 的取 git diff -U0（工作区对暂存区）的加行。
"""
import os, re, subprocess, sys

MINIMUM = 8
INFO_LESS = re.compile(r'^(fi|done|esac|then|else|do|in|;;|PY|EOF|pass|continue|break|return|else:|try:|finally:|[(){}\[\],;]+)$')
SHORT = 8

def normalized(path, quotes):
    out = []
    try:
        lines = open(path, encoding='utf-8', errors='replace').read().split('\n')
    except OSError:
        return out
    for number, line in enumerate(lines, 1):
        text = ' '.join(line.split())
        if not text or text.startswith('#'):
            continue
        if len(text) < SHORT or INFO_LESS.match(text):
            continue
        if quotes:
            text = text.replace("'", '"')
        out.append((number, text))
    return out

def added_lines(root, path, kind):
    if kind == 'new':
        return None   # 全部算加行
    diff = subprocess.run(['git', '-C', root, 'diff', '-U0', '--', path], capture_output=True, text=True).stdout
    numbers = set()
    for hunk in re.finditer(r'^@@ -\S+ \+(\d+)(?:,(\d+))? @@', diff, re.M):
        start, count = int(hunk.group(1)), int(hunk.group(2) or '1')
        numbers.update(range(start, start + count))
    return numbers

def main():
    root, listing = sys.argv[1], sys.argv[2]
    os.chdir(root)
    judged = [line.split('\t') for line in open(listing, encoding='utf-8').read().split('\n') if line.strip() and not line.startswith('#')]
    files = subprocess.run(['git', 'ls-files', '-co', '--exclude-standard', '--', '.claude/gate.d', '.claude/hooks', '.claude/scripts',
                            '.claude/singlefs-ai-sop/scripts', 'scripts'], capture_output=True, text=True).stdout.split('\n')
    files = sorted({f for f in files if f.endswith(('.sh', '.py')) and '/fixtures/' not in f and os.path.isfile(f)})
    for quotes in (False, True):
        windows = {}
        cache = {}
        for f in files:
            lines = normalized(f, quotes)
            cache[f] = lines
            for position in range(len(lines) - MINIMUM + 1):
                windows.setdefault(tuple(t for _n, t in lines[position:position + MINIMUM]), []).append((f, position))
        print(f'== 档：{"引号归一" if quotes else "逐字"}；对照 {len(files)} 份')
        total = 0
        for path, kind in judged:
            lines = cache.get(path) or normalized(path, quotes)
            added = added_lines(root, path, kind)
            hits = {}
            for position in range(len(lines) - MINIMUM + 1):
                window = lines[position:position + MINIMUM]
                if added is not None and not any(n in added for n, _t in window):
                    continue
                for other, other_position in windows.get(tuple(t for _n, t in window), []):
                    if other == path and abs(other_position - position) < MINIMUM:
                        continue
                    hits.setdefault(other, []).append((position, other_position))
            for other, pairs in sorted(hits.items()):
                pairs.sort()
                runs, start, previous = [], None, None
                for position, other_position in pairs:
                    if previous and position == previous[0] + 1 and other_position == previous[1] + 1:
                        previous = (position, other_position)
                        continue
                    if start:
                        runs.append((start, previous))
                    start = previous = (position, other_position)
                runs.append((start, previous))
                other_lines = cache.get(other) or normalized(other, quotes)
                for (s, so), (e, _eo) in runs:
                    total += 1
                    length = e - s + MINIMUM
                    print(f'{path}:{lines[s][0]}-{lines[e + MINIMUM - 1][0]} 与 {other}:{other_lines[so][0]} 起，连续 {length} 行相同')
        print(f'== 小计：{total} 段')

main()
