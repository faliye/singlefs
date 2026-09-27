#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 状态一致性：状态别说两遍，也别挂错节（小节标题不带「—— 已定」、条目不同时写破折号与「状态：」、分项索引行的「状态：」与所在节一致）
#
# 分项已经按状态分住「### 已定项」与「### 未定项」两节 ⇒ **节本身就是状态**。
# 再在标题或条目里写一遍「—— 已定」，同一个词就说了两遍；
# 而重复的标注会各自漂移——检索取到其中一处时不知道另一处写的是什么
# （`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条：矛盾比空白更糟）。
#
# 判三样：
#   1. 小节标题不许写成「已定项 N —— 已定…」（节名已经说过了；编号后面带括注的也算）
#   2. 同一个条目不许既有「—— 已定 / 未定」又有「状态：已定 / 未定」，两处的值相不相同都判
#   一份都没扫到、或一条分项索引行都没核到，退 77（不记通过）。
#   3. 条目的「状态：」必须与它所在的节一致（已定项节里不许有状态：未定）
#   2、3 只看每节的索引（第一个 `####` 之前），与 20-kb-shape.sh 第 5 段同一条边界；
#   `####` 之下的论证里另有编号列表，不是分项。
#
# 判别力：fixtures/24-status-redundancy.sh/red 三样各犯一次，必须判红；
# green 里另放一份论证带「3. …状态：未定」的决策，必须判绿。
#
# ⚠️ 只认**显式的**「状态：X」与破折号形态，不认句中随口提到的「已定」——
#    一条未定项里出现「D18 已定项 3 已定」是正常引用，不是它自己的状态。
#
#   bash .claude/gate.d/24-status-redundancy.sh
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
DEC=.claude/kb/decisions
[[ -d "$DEC" ]] || { echo "  ! 找不到 $DEC，本阶段跳过"; exit 77; }

# 扫的范围：决策正文 + 实验正文 + records/。
# 后两处今天是干净的（2026-08-31 现查：三类毛病各 0 处），扫它们是**防复发**——
# 一条只在出事之后才加的检查，等于承认那一次出事没人拦得住。
python3 - "$DEC" .claude/kb/experiments records <<'PY'
import re, sys, glob, os
bad = []
files = []
rows_checked = 0
for d in sys.argv[1:]:
    if os.path.isdir(d):
        files += sorted(glob.glob(os.path.join(d, '**', '*.md'), recursive=True))

for f in files:
    body = open(f, encoding='utf-8').read().split('\n## 历史版本')[0]
    name = os.path.basename(f)

    # 1. 标题里重复
    for i, line in enumerate(body.split('\n'), 1):
        # 编号后面许跟一段括注（「已定项 3（2026-09-01）—— 已定」），不许的话带日期的标题整个漏过去
        if re.match(r'^#{3,4} (已定项|未定项) \d+ *(?:[（(][^）)]*[）)])? *—— *\*{0,2}(已定|未定)', line):
            bad.append((name, i, '标题重复', line.strip()[:60]))

    # 2 / 3. 条目层
    # 只看每节的**索引**：节到下一个一至三级标题为止，再在第一个 `####` 处收口——
    # `####` 之下是各分项自己的论证，里面另有编号列表（「3. …状态：未定」这类），那些不是分项。
    # 边界与 20-kb-shape.sh 第 5 段的 index_of、lib-index-vs-body.py 的 count_items 同一条。
    for sec_name, want in (('已定项', '已定'), ('未定项', '未定')):
        m = re.search(r'^### %s\s*$(.*?)(?=^#{1,3} |\Z)' % sec_name, body, re.M | re.S)
        if not m:
            continue
        section_text = m.group(1)
        first_subheading = re.search(r'^#{4}\s', section_text, re.M)
        index_text = section_text[:first_subheading.start()] if first_subheading else section_text
        base = body[:m.start(1)].count('\n') + 1
        for off, line in enumerate(index_text.split('\n')):
            if not re.match(r'^(?:\d+\.|\|\s*\d+\s*\|)', line):
                continue
            rows_checked += 1
            ln = base + off
            dash = re.search(r'——\s*\*{0,2}(已定|未定)', line)
            stat = re.search(r'状态：\s*\*{0,2}(已定|未定)', line)
            # 破折号与「状态：」同时出现就算说了两遍，两处写的值相不相同都一样：值不同时是两处已经漂开了
            if dash and stat:
                bad.append((name, ln, '条目说两遍', line.strip()[:60]))
            if stat and stat.group(1) != want:
                bad.append((name, ln, '挂错节', line.strip()[:60]))

if bad:
    print('  ✗ 状态被说了两遍，或分项挂错了节：')
    for n, ln, why, txt in bad[:20]:
        print(f'     {n}:{ln}  {why} —— {txt}')
    if len(bad) > 20:
        print(f'     …… 另有 {len(bad)-20} 处')
    print('     → 小节标题写成「### 已定项 N（日期）：结论」，别再写「—— 已定」；')
    print('     → 条目二选一：留「状态：已定」这一处机器可读的标注，把「—— 已定」去掉，')
    print('       日期与结论一个字都不要丢；')
    print('     → 状态与所在节不一致的，把条目搬到对的那一节，别就地改状态词。')
    sys.exit(1)

# 扫到 0 份、或一条分项索引行都没核到：条目层的 2、3 没有对象，退 77，不记通过
# （.claude/singlefs-ai-sop/rules/show-me-test.md「扫到 0 项也不是通过」）。
if not files or rows_checked == 0:
    print(f'  ! 本次无对象可判：扫了 {len(files)} 份、核到 {rows_checked} 条分项索引行（决策正文、实验正文、records/ 下的 .md 里没有分项索引）')
    sys.exit(77)
print(f'  ✓ 状态只说一遍，且分项都在对的节里（扫 {len(files)} 份、核 {rows_checked} 条分项索引行）')
PY
