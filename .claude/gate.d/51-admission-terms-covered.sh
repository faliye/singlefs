#!/usr/bin/env bash
# gate-stage: 准入不等式的每一项都有人维护（统计量，或写明的例外）
#
# 判据（2026-09-13 用户定案，登记在 D5（快照 / 空间记账机制） 已定项 4）：
# 「式子里出现的每一项都必须是被维护的统计量，如果无法做到就写上例外是谁」。
#
# 做法：
#   1. 式子的权威正文前面有一行 `<!-- gate:admission-formula -->`，全仓只许一处；取它下面第一个代码块里 `可用 =` 那一行，
#      按 `−` 拆成项（去掉 `Σ设备(` 与括号）。
#   2. 对照表前面有一行 `<!-- gate:admission-terms -->`，全仓只许一处；取它下面第一张表的两列。
#   3. 两边的项必须逐一相等——式子加了一项而对照表没加、或对照表多出一项，都判红。
#   4. 对照表每一行的右列要么是「第 N 项（名字）」，N 在同一文件那张 `| # | 统计量 |` 表里、名字逐字相等、且不是已撤回的编号；
#      要么以 `**例外**：` 开头并写出理由（不少于 8 个字）。
#   5. `decisions/` 下凡是写着 `可用 = Σ设备` 的地方（索引行、挪走后留的指针），去掉空白与反引号之后必须与权威那一行逐字相等。
#
# ⚠️ **这条是实测出来的**：2026-09-13 立 D28（挂载期承诺量） 时往式子里加了第五项「挂载期承诺量」，
# 而 D5（快照 / 空间记账机制） 已定项 4 的完备性口径逐字是「式子里出现的每一项都必须是被维护的统计量」——
# 新项只住内存、不是统计量，那句话当场为假；同一轮回头看，式子里的「容量」也从来不是统计量，没有任何东西发现过。
#
# 射程（`.claude/singlefs-ai-sop/rules/show-me-test.md`「没实现的要明说」）：
#   只判「每一项都有归属、归属指得到」，不判归属对不对（某一项该不该是统计量、例外的理由成不成立，靠人）；
#   不判统计量表里多出来的项（那一半是「没出现在任何判定里的就不要」，判定不止准入一处，机器列不全）。
#
#   bash .claude/gate.d/51-admission-terms-covered.sh [仓根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
[[ -d .claude/kb/decisions ]] || { echo "  ! 找不到 .claude/kb/decisions/，本阶段跳过"; exit 0; }

python3 - <<'PY'
import glob, re, sys

files = sorted(glob.glob('.claude/kb/decisions/*.md'))
texts = {path: open(path, encoding='utf-8').read() for path in files}

def anchors(marker):
    hits = []
    for path, text in texts.items():
        lines = text.split('\n')
        for number, line in enumerate(lines):
            if line.strip() == marker:
                hits.append((path, number, lines))
    return hits

problems = []

formula_hits = anchors('<!-- gate:admission-formula -->')
terms_hits = anchors('<!-- gate:admission-terms -->')
if len(formula_hits) != 1 or len(terms_hits) != 1:
    print(f'  ✗ 准入式子的锚 {len(formula_hits)} 处、对照表的锚 {len(terms_hits)} 处，各要恰好 1 处')
    print('     → 在准入不等式权威正文的代码块上一行写 `<!-- gate:admission-formula -->`，')
    print('       在逐项对照表上一行写 `<!-- gate:admission-terms -->`；两个锚全仓各一处，别处只留指针。')
    sys.exit(1)

formula_path, formula_line, formula_lines = formula_hits[0]
formula_text = None
inside_block = False
for line in formula_lines[formula_line + 1:]:
    if line.startswith('```'):
        if inside_block:
            break
        inside_block = True
        continue
    if inside_block and line.strip().startswith('可用 ='):
        formula_text = line.strip()
        break
if formula_text is None:
    print(f'  ✗ {formula_path}:{formula_line + 1} 的锚下面没有代码块，或代码块里没有 `可用 =` 那一行')
    print('     → 锚要紧挨着式子的代码块，式子写成一行：可用 = Σ设备( … ) − … 。')
    sys.exit(1)

right_hand_side = formula_text.split('=', 1)[1]
right_hand_side = right_hand_side.replace('Σ设备(', ' ').replace('Σ设备（', ' ').replace('(', ' ').replace(')', ' ').replace('（', ' ').replace('）', ' ')
formula_terms = [term.strip() for term in right_hand_side.split('−') if term.strip()]

terms_path, terms_line, terms_lines = terms_hits[0]
mapping = {}
table_started = False
for line in terms_lines[terms_line + 1:]:
    stripped = line.strip()
    if not stripped:
        if table_started:
            break
        continue
    if not stripped.startswith('|'):
        if table_started:
            break
        continue
    table_started = True
    cells = [cell.strip() for cell in stripped.strip('|').split('|')]
    if len(cells) < 2 or set(cells[0]) <= set('-: ') or cells[0] == '式子里的项':
        continue
    mapping[cells[0]] = cells[1]

statistics = {}
statistics_text = texts[terms_path].split('\n')
in_statistics_table = False
for line in statistics_text:
    if re.match(r'^\| # \| 统计量 \|', line):
        in_statistics_table = True
        continue
    if in_statistics_table:
        if not line.startswith('|'):
            in_statistics_table = False
            continue
        cells = [cell.strip() for cell in line.strip().strip('|').split('|')]
        if cells and cells[0].isdigit():
            statistics[int(cells[0])] = cells[1].replace('**', '')

if not statistics:
    problems.append(f'{terms_path}: 找不到表头是 `| # | 统计量 |` 的统计量表，对照表的「第 N 项」没处可指')

formula_set, mapping_set = set(formula_terms), set(mapping)
for term in formula_terms:
    if term not in mapping_set:
        problems.append(f'式子里有「{term}」，对照表（{terms_path}）里没有这一行')
for term in mapping:
    if term not in formula_set:
        problems.append(f'对照表（{terms_path}）里有「{term}」，式子（{formula_path}）里没有这一项')

exceptions = []
for term, owner in mapping.items():
    if owner.startswith('**例外**：'):
        reason = owner[len('**例外**：'):].strip()
        if len(reason) < 8:
            problems.append(f'「{term}」写了例外，但理由不到 8 个字：{owner}')
        exceptions.append(term)
        continue
    match = re.match(r'^第 (\d+) 项（(.+)）$', owner)
    if not match:
        problems.append(f'「{term}」的归属写成了「{owner}」，要么「第 N 项（名字）」、要么「**例外**：理由」')
        continue
    number, name = int(match.group(1)), match.group(2)
    if number not in statistics:
        problems.append(f'「{term}」指到第 {number} 项，统计量表里没有这一行')
    elif '已撤回' in statistics[number]:
        problems.append(f'「{term}」指到第 {number} 项，而那一项已撤回')
    elif statistics[number] != name:
        problems.append(f'「{term}」写的是第 {number} 项（{name}），统计量表里第 {number} 项叫「{statistics[number]}」')

canonical = re.sub(r'[\s`]', '', formula_text)
copies = 0
for path, text in texts.items():
    for number, line in enumerate(text.split('\n'), 1):
        position = line.find('可用 = Σ设备')
        if position < 0:
            position = line.find('可用 =Σ设备')
        if position < 0:
            continue
        copies += 1
        fragment = line[position:]
        fragment = fragment.split('`')[0] if '`' in line[:position] else fragment
        fragment = re.sub(r'[\s`]', '', fragment)
        if not fragment.startswith(canonical):
            problems.append(f'{path}:{number} 的式子副本与权威那一行（{formula_path}）对不上')

if problems:
    print(f'  ✗ 准入不等式与统计量清单的逐项对照有 {len(problems)} 处不对')
    for problem in problems:
        print(f'     {problem}')  # gate-lint:detail
    print('     → 式子里每一项都要在对照表里有一行：被维护的统计量写「第 N 项（名字）」，做不到的写「**例外**：理由」')
    print('       （2026-09-13 用户定案，D5（快照 / 空间记账机制） 已定项 4）；式子改了，先改对照表，再改各处副本。')
    sys.exit(1)

print(f'  ✓ 准入不等式 {len(formula_terms)} 项逐项有归属：统计量 {len(formula_terms) - len(exceptions)} 项，写明的例外 {len(exceptions)} 项（{"、".join(exceptions)}）；式子副本 {copies} 处与权威一致')
PY
