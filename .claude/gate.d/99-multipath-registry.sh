#!/usr/bin/env bash
# gate-stage: 多条路径互证的实验，路径与共用项要登记且指得到
#
# 还 checks-owed.md C48（多条校验路径共用同一个前提）可机检的那一半。
#
# 判据与形态在 `.claude/rules/format-evolution.md`「实验页的「路径与结论登记」」，这一道判三条：
#   ① 登记表的形状：`### 路径与结论登记` 之下恰好一张表，表头逐字
#      `| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |`，每行五格；
#   ② 源码落点指得到：写成 `路径:行号` 的，那个文件要在仓里现存；只活在正文里的写
#      「无实现：理由」，理由不许空；
#   ③ 共用项那一格不许空，也不许只写「无」——多条路径逐格相等不构成证据，当它们共用
#      同一个错误前提时，而那一列正是唯一能让人看出「共用了什么」的地方。
#
# ⚠️ **射程只到已经写了这一节的实验页**。仓里今天 10 份实验页在做多条路径互证（普查
# `research/prompts/c48-c49-multipath-survey.md`），只有 1 份写了登记节；其余 9 份登记在
# `.claude/gate.d/multipath-registry-lag.tsv`，那张表**只缩不涨**：一份补齐了就删一行，
# 新写的实验页不许往里加。要判「该写而没写」那一半得先认出「这个实验在做多条路径互证」，
# 而按关键词扫正文认不出来——普查里 15 份命中有 5 份，grep 命中的那一行说的是别的意思，
# 真结构在页面别处。那一半的欠账在 C48（多条校验路径共用同一个前提）。
#
# 判别力：fixtures/99-multipath-registry.sh/{red,green}/ 各一份实验页，
# red 那份三行里一行共用项写「无」、一行源码落点指不到文件，必须判红。
set -uo pipefail
cd "${1:-$(dirname "$0")/../..}" 2>/dev/null || true
EXPERIMENTS=.claude/kb/experiments
LAG=.claude/gate.d/multipath-registry-lag.tsv
[[ -d "$EXPERIMENTS" ]] || { echo "  ! 没有 $EXPERIMENTS，本阶段无对象可判"; exit 7; }

python3 - "$EXPERIMENTS" "$LAG" <<'PY'
import os, re, sys, glob

experiments, lag_path = sys.argv[1], sys.argv[2]
HEADING = '### 路径与结论登记'
WANTED = '| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |'

lagging = set()
if os.path.isfile(lag_path):
    for line in open(lag_path, encoding='utf-8'):
        line = line.strip()
        if line and not line.startswith('#'):
            lagging.add(line.split('\t')[0])

problems, checked_pages, checked_rows = [], 0, 0
for path in sorted(glob.glob(os.path.join(experiments, '*.md'))):
    text = open(path, encoding='utf-8').read()
    if HEADING not in text:
        continue
    checked_pages += 1
    section = text.split(HEADING, 1)[1]
    cut = re.search(r'^#{2,4}\s', section, re.M)
    if cut:
        section = section[:cut.start()]
    rows = [line for line in section.split('\n') if line.startswith('|')]
    if not rows:
        problems.append(f"{path}：「{HEADING}」之下一张表都没有")
        continue
    if rows[0].strip() != WANTED:
        problems.append(f"{path}：表头不是 {WANTED}，实际是 {rows[0].strip()[:60]}")
        continue
    for row in rows[2:]:
        cells = [cell.strip() for cell in row.strip().strip('|').split('|')]
        if len(cells) != 5:
            problems.append(f"{path}：这一行不是五格：{row.strip()[:60]}")
            continue
        checked_rows += 1
        number, name, _how, where, shared = cells
        for hit in re.findall(r'`([^`]+?):(\d+)`', where):
            if not os.path.exists(hit[0]):
                problems.append(f"{path}：第 {number} 行「{name}」的源码落点指不到文件：{hit[0]}")
        if not re.search(r'`[^`]+?:\d+`', where) and not where.startswith('无实现：'):
            problems.append(f"{path}：第 {number} 行「{name}」的源码落点既不是 路径:行号，也不是「无实现：理由」：{where[:40]}")
        if where.startswith('无实现：') and not where[len('无实现：'):].strip():
            problems.append(f"{path}：第 {number} 行「{name}」写了「无实现：」而理由是空的")
        if not shared or shared in ('无', '—', '-', '无。'):
            problems.append(f"{path}：第 {number} 行「{name}」的共用项那一格是空的或只写了「{shared}」")

if problems:
    print(f"  ✗ {len(problems)} 处登记不合形态：")                       # gate-lint:summary
    for entry in problems:
        print(f"     {entry}")                                          # gate-lint:detail
    print("     → 怎么办：形态见 .claude/rules/format-evolution.md「实验页的「路径与结论登记」」。")
    print("               表头逐字 | # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |，每行五格；")
    print("               源码落点写 `路径:行号 函数名`，只活在正文里的写「无实现：理由」；")
    print("               共用项写这条路径读了哪些常量、哪些规则、哪份规范，一个都不许省——")
    print("               多条路径逐格相等不构成证据，当它们共用同一个错误前提时。")
    sys.exit(1)

print(f"  ✓ 登记了路径与共用项的实验页判过了（{checked_pages} 份、{checked_rows} 条路径）；"
      f"还没写登记节的 {len(lagging)} 份在 {os.path.basename(lag_path)} 里，逐份补齐后删行")
PY
