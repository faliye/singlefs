#!/usr/bin/env bash
# gate-stage: 编号与简称在 doc-lint 够不到的地方也要一致
#
# `doc-lint.sh` 只管 kb 与 rules 下的 markdown。而编号引用还散在
# **实验源码的注释、research/scripts、records/** 里——那些地方没有任何东西在看。
# 实测（2026-08-31）：扫出 8 处不符，含 3 处把全角「）」写成半角「)」、
# 1 处把 D4 的未定项名字当成了 D4 的简称。
#
# 为什么判红：编号在引用处只剩一个符号，含义能被悄悄改掉而没有一个字看起来别扭
# （`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 5 条）。
#
# ⚠️ **`research/prompts/` 显式排除**，理由不是懒：那是当时**原样发给模型的提示**，
# 与 `research/results/` 里的产物一一对应。事后改提示 = 产物不再对应它的输入，
# 那正是 C38（外部文献不可复核）那一类「证据蒸发」。
# 提示里的旧简称是**当时写的**，要改只能连同重跑一起改。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
[[ -d .claude/kb/decisions ]] || { echo "  ! 找不到 kb，本阶段跳过"; exit 0; }

python3 - <<'PY'
import re, glob, os, sys

truth = {}
for pat, pre in (('.claude/kb/experiments/*.md', 'E'), ('.claude/kb/decisions/*.md', 'D')):
    for f in glob.glob(pat):
        t = open(f, encoding='utf-8').read().split('\n', 1)[0]
        m = re.match(r'## (' + pre + r'\d+) (.+?)\s*——', t)
        if m:
            truth[m.group(1)] = m.group(2).strip()

targets = []
for pat in ('research/**/*.rs', 'research/scripts/*.sh', 'records/**/*.md'):
    targets += glob.glob(pat, recursive=True)
targets = [f for f in targets if '/prompts/' not in f]

bad = []
for f in sorted(set(targets)):
    for i, line in enumerate(open(f, encoding='utf-8', errors='ignore'), 1):
        for num, nm in re.findall(r'([ED]\d+)（([^）]*)）', line):
            if num in truth and truth[num] != nm:
                bad.append((f, i, num, nm, truth[num]))

if bad:
    print('  ✗ 编号的简称与登记位不符：')
    for f, i, num, nm, want in bad[:15]:
        print(f'     {f}:{i}  {num} 写的「{nm[:24]}」，登记处是「{want[:24]}」')
    if len(bad) > 15:
        print(f'     …… 另有 {len(bad)-15} 处')
    print('     → 简称照登记位抄（各正文首行 `## D<n> 简称 —— 状态`）。')
    print('     → 若是「）」写成了半角「)」，括注不闭合，正则会一路吃到下一个右括号。')
    print('     → research/prompts/ 不在扫描范围：那是原样发给模型的提示，改它等于让产物对不上输入。')
    sys.exit(1)

n = len(set(targets))
print(f'  ✓ 编号与简称一致（扫 {n} 个文件，prompts/ 按证据链原样保留）')
PY
