#!/usr/bin/env bash
# gate-stage: 决策索引结论列的宽度
#
# `decisions.md` 的索引表是检索这批决策的入口，「结论」列只写那条决策**定了什么**。
# 表头上写着一个字数上限，而这个上限此前没有任何东西在执行——实测（2026-09-07）
# 七行超标、最长 96 字，当时写的限是 50。一条只写在文档里、没人执行的规约，
# 下一个人会照着最长的那行写，索引就退化成第二份正文，导航价值归零。
#
# 上限**从 decisions.md 那一行读**，不写死在这里：同一个事实只许一处权威记录
# （`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条）。改限只改文档那一行，
# 本阶段跟着走；两边各写各的，就会一处改对、一处停在旧值。
#
#   bash .claude/gate.d/37-decision-summary-width.sh
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
IDX=.claude/kb/decisions.md
[[ -f "$IDX" ]] || { echo "  ! 找不到 $IDX，本阶段跳过"; exit 0; }

python3 - "$IDX" <<'PY'
import re, sys

path = sys.argv[1]
text = open(path, encoding="utf-8").read()

m = re.search(r"「结论」列一律\s*(\d+)\s*字以内", text)
if not m:
    print("  ✗ decisions.md 的索引表说明里找不到「「结论」列一律 N 字以内」")   # gate-lint:summary
    print("  → 怎么办：上限的权威记录是那一句，本阶段从它读。把它写回索引表的说明里，")
    print("    例：⚠️ **「结论」列一律 200 字以内**，写那条决策定了什么。")
    sys.exit(1)
limit = int(m.group(1))

rows, bad = [], []
for line in text.splitlines():
    if not re.match(r"^\|\s*D\d+（", line):
        continue
    cells = [c.strip() for c in line.strip().strip("|").split("|")]
    if len(cells) < 4:
        continue
    name = cells[0]
    concl = re.sub(r"\s+", " ", cells[2])
    rows.append(name)
    if not concl:
        bad.append(f"{name} 的结论列是空的")
    elif len(concl) > limit:
        bad.append(f"{name} 的结论列 {len(concl)} 字，超出上限 {limit} 字：{concl[:36]}…")

if not rows:
    print("  ✗ decisions.md 里一行 `| D<n>（简称） | 状态 | 结论 | 正文 |` 都没扫到")   # gate-lint:summary
    print("  → 怎么办：索引表的行首形态是 `| D1（数据可移动性 / 反向索引） | …`。")
    print("    改过表格形状就同步改本阶段的匹配，别让它扫到 0 行还报绿。")
    sys.exit(1)

if bad:
    print(f"  ✗ 决策索引结论列有 {len(bad)} 行超出上限（限 {limit} 字）：")   # gate-lint:summary
    for b in bad:
        print("      " + b)                                                  # gate-lint:detail
    print("  → 怎么办：结论列只写那条决策**定了什么**，为什么定、还欠什么一律搬进 decisions/ 下的正文。")
    print("    删不动就说明这条决策的主结论本身没收敛——那要么改它的状态，要么把分项拆开写。")
    sys.exit(1)

print(f"  ✓ 决策索引结论列都不超 {limit} 字（共 {len(rows)} 行）")
PY
