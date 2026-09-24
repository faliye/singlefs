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
#      同一个错误前提时，而那一列正是唯一能让人看出「共用了什么」的地方；
#   ④ 还没写登记节的实验页登记在 `.claude/gate.d/multipath-registry-lag.tsv`，这张表每一行：
#      两列（页面路径、普查判定的依据）用制表符分隔、都不许空；路径形如
#      `.claude/kb/experiments/<编号>-….md` 且文件在仓里现存；那一页确实还没有登记节——补上了就判过期，删行；
#      **只缩不涨**：与基准提交比，基准里这张表没有的实验编号不许出现。按编号比、不按整条路径比：
#      页面改名、登记行抄错了题目改正，都不算新加了一份。
#
# ⚠️ **射程只到已经写了这一节的实验页**。建表时 10 份实验页在做多条路径互证（普查
# `research/prompts/c48-c49-multipath-survey.md`，已归档，去 git 历史里找：
# `git show 13bccea:research/prompts/c48-c49-multipath-survey.md`），只有 1 份写了登记节；
# 其余登记在滞后表里，那张表**只缩不涨**：一份补齐了就删一行，新写的实验页不许往里加。
# 要判「该写而没写」那一半得先认出「这个实验在做多条路径互证」，
# 而按关键词扫正文认不出来——普查里 15 份命中有 5 份，grep 命中的那一行说的是别的意思，
# 真结构在页面别处。那一半的欠账在 C48（多条校验路径共用同一个前提）。
#
# 判别力：fixtures/99-multipath-registry.sh/{red,green}/ 各一份实验页，
# red 那份三行里一行共用项写「无」、一行源码落点指不到文件，另带一张滞后表：一行指向不存在的页、
# 一行指向已经写了登记节的页、一行少了依据那一列、一行的实验编号基准里没有，必须判红；
# green 的滞后表只有一行、指向一份还没写登记节的页、与基准相同，必须判绿。
set -uo pipefail
cd "${1:-$(dirname "$0")/../..}" 2>/dev/null || true
EXPERIMENTS=.claude/kb/experiments
LAG=.claude/gate.d/multipath-registry-lag.tsv
[[ -d "$EXPERIMENTS" ]] || { echo "  ! 没有 $EXPERIMENTS，本阶段无对象可判"; exit 77; }
base=""
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  # 基准取法与 56、68、69、75、97 号同一份：research/scripts/changed-paths.sh 的 gate 取法
  LIB_CHANGED_PATHS="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh"
  # shellcheck source=../../research/scripts/changed-paths.sh
  source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用脚本 $LIB_CHANGED_PATHS"; echo "     → 怎么办：它随仓走（research/scripts/changed-paths.sh），被删了就从 git 找回来。"; exit 1; }
  base="$(gate_diff_base gate)"
fi

python3 - "$EXPERIMENTS" "$LAG" "$base" <<'PY'
import os, re, subprocess, sys, glob

experiments, lag_path, base = sys.argv[1], sys.argv[2], sys.argv[3]
HEADING = '### 路径与结论登记'
WANTED = '| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |'
LAG_PAGE = re.compile(r'\.claude/kb/experiments/(?P<number>[0-9]+)-[^/]+\.md')

def has_registration(text):
    return HEADING in text

def lag_rows(text):
    """滞后表的内容行：[(行号, 各列)]，注释与空行不算。"""
    rows = []
    for line_number, line in enumerate(text.split('\n'), 1):
        if line.strip() and not line.startswith('#'):
            rows.append((line_number, line.rstrip('\r').split('\t')))
    return rows

problems, checked_pages, checked_rows = [], 0, 0
for path in sorted(glob.glob(os.path.join(experiments, '*.md'))):
    text = open(path, encoding='utf-8').read()
    if not has_registration(text):
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

# ④ 滞后表：形状、路径、过期、只缩不涨
lag_problems, lagging, lag_pages_by_number = [], [], {}
if os.path.isfile(lag_path):
    for line_number, fields in lag_rows(open(lag_path, encoding='utf-8').read()):
        where = f"{os.path.basename(lag_path)} 第 {line_number} 行"
        if len(fields) != 2 or not all(field.strip() for field in fields):
            lag_problems.append(f"{where}：不是两列（页面路径、普查判定的依据）或有一列是空的：{chr(9).join(fields)[:80]}")
            continue
        page = fields[0].strip()
        page_shape = LAG_PAGE.fullmatch(page)
        if not page_shape:
            lag_problems.append(f"{where}：路径不是 .claude/kb/experiments/<编号>-….md 的形状：{page}")
            continue
        lag_pages_by_number[int(page_shape.group('number'))] = page
        if not os.path.isfile(page):
            lag_problems.append(f"{where}：指向的实验页不存在：{page}")
            continue
        if has_registration(open(page, encoding='utf-8').read()):
            lag_problems.append(f"{where}：{page} 已经写了「{HEADING}」，这一行过期了")
            continue
        lagging.append(page)

baseline_numbers = None
if base:
    shown = subprocess.run(['git', '-c', 'core.quotepath=false', 'show', f'{base}:{lag_path}'],
                           capture_output=True, text=True)
    if shown.returncode == 0:
        baseline_numbers = set()
        for _line_number, fields in lag_rows(shown.stdout):
            page_shape = LAG_PAGE.fullmatch(fields[0].strip())
            if page_shape:
                baseline_numbers.add(int(page_shape.group('number')))
grown = sorted(set(lag_pages_by_number) - baseline_numbers) if baseline_numbers is not None else []

if problems:
    print(f"  ✗ {len(problems)} 处登记不合形态：")                       # gate-lint:summary
    for entry in problems:
        print(f"     {entry}")                                          # gate-lint:detail
    print("     → 怎么办：形态见 .claude/rules/format-evolution.md「实验页的「路径与结论登记」」。")
    print("               表头逐字 | # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |，每行五格；")
    print("               源码落点写 `路径:行号 函数名`，只活在正文里的写「无实现：理由」；")
    print("               共用项写这条路径读了哪些常量、哪些规则、哪份规范，一个都不许省——")
    print("               多条路径逐格相等不构成证据，当它们共用同一个错误前提时。")
if lag_problems:
    print(f"  ✗ 滞后表 {lag_path} 里 {len(lag_problems)} 行对不上仓里的实验页：")  # gate-lint:summary
    for entry in lag_problems:
        print(f"     {entry}")                                          # gate-lint:detail
    print("     → 怎么办：路径抄错了就照 .claude/kb/experiments/ 里现存的文件名改；那一页已经补了登记节，就删掉这一行——")
    print("               留着的滞后行会让人以为那一页还欠着，而指向不存在的页等于那一格没人管。")
if grown:
    print(f"  ✗ 滞后表涨了：这些实验页的编号，基准 {base} 的滞后表里没有：")  # gate-lint:summary
    for number in grown:
        print(f"     {lag_pages_by_number[number]}")                     # gate-lint:detail
    print("     → 怎么办：这张表只缩不涨。新写的实验页要做多条路径互证，就当场写「### 路径与结论登记」那一节，")
    print("               不许加一行滞后把它按下去——加得了一行，这一道对新写的实验页就什么都拦不住了。")
if problems or lag_problems or grown:
    sys.exit(1)

compared = f"与基准 {base} 比没涨" if baseline_numbers is not None else "基准里还没有这张表（或不是 git 仓），只缩不涨这一条没比"
print(f"  ✓ 登记了路径与共用项的实验页判过了（{checked_pages} 份、{checked_rows} 条路径）；"
      f"还没写登记节的 {len(lagging)} 份在 {os.path.basename(lag_path)} 里、每一份都现存且确实还没写，{compared}；逐份补齐后删行")
PY
