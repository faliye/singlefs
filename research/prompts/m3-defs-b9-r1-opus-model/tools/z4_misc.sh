#!/usr/bin/env bash
# Z4 零散三样：
#   rform    出处那一行写着一个仓里没有材料的 <名>-r<数字>（ghost-r2）：各版判法收不收（M4 把 -r 形认成别的轮名，两份文件头仍写「不按 -r<数字> 形」）
#   drift    攻方试改 F-Z4 在第三轮 Y1-丙 那种插行漂移上还收不收，插行之后再改那一行的字还拒不拒
#   legacy   旧行名单里已没有登记行的名字（c196-r1）新登记一行不带出处：收不收
# 用法：z4_misc.sh <工作目录> <名>=<副本根>…（判法用各副本自己的 .claude/hooks/lib_abandoned_rounds.py）
set -euo pipefail
work="$1"; shift
mkdir -p "$work"
judge() { ( cd "$1" && PYTHONDONTWRITEBYTECODE=1 python3 -B "$2/.claude/hooks/lib_abandoned_rounds.py" . 2>&1 ) | tr '\n' ' ' || true; }
tree="$work/rform"; mkdir -p "$tree/records" "$tree/research/prompts/hx-form-r1-snapshot"; : > "$tree/research/prompts/_hx-form-r1-body.md"
printf '%s\n' '# 调度记录' '' '## 2026-10-12' '' '- 开轮 hx-form-r1。' '- 用户「这一轮先停掉，不写判决了」。去向：hx-form-r1 两条腿当场停掉，改并进 ghost-r2 那一题（ghost-r2 还没开轮、仓里没有材料）' > "$tree/records/log.md"
printf 'hx-form-r1\t2026-10-12\t用户叫停（出处 records/log.md 第 6 行〔去向：hx-form-r1 两条腿当场停掉〕）\t并进别处\n' > "$tree/research/prompts/abandoned-rounds.tsv"
for entry in "$@"; do printf 'rform\t%s\t%s\n' "${entry%%=*}" "$(judge "$tree" "${entry#*=}")"; done
tree="$work/drift"; mkdir -p "$tree/records" "$tree/research/prompts/hx-drift-r1-snapshot"; : > "$tree/research/prompts/_hx-drift-r1-body.md"
printf '%s\n' '# 调度记录' '' '## 2026-10-12' '' '- 开轮 hx-drift-r1。' '- 用户「这一轮先停掉，不写判决了」。去向：hx-drift-r1 两条腿当场停掉' > "$tree/records/log.md"
printf 'hx-drift-r1\t2026-10-12\t用户叫停（出处 records/log.md 第 6 行〔去向：hx-drift-r1 两条腿当场停掉〕）\t不写判决\n' > "$tree/research/prompts/abandoned-rounds.tsv"
for entry in "$@"; do printf 'drift\t插行前\t%s\t%s\n' "${entry%%=*}" "$(judge "$tree" "${entry#*=}")"; done
sed -i '2a ### 新插的一节' "$tree/records/log.md"
for entry in "$@"; do printf 'drift\t插行后\t%s\t%s\n' "${entry%%=*}" "$(judge "$tree" "${entry#*=}")"; done
sed -i '7s/当场停掉/当场停了/' "$tree/records/log.md"
for entry in "$@"; do printf 'drift\t插行又改字\t%s\t%s\n' "${entry%%=*}" "$(judge "$tree" "${entry#*=}")"; done
tree="$work/legacy"; mkdir -p "$tree/records" "$tree/research/prompts"
printf 'c196-r1\t2026-10-13\t没有出处的新登记（名字碰巧是旧行名单里的）\t无\nhx-new-r1\t2026-10-13\t没有出处的新登记\t无\n' > "$tree/research/prompts/abandoned-rounds.tsv"
for entry in "$@"; do printf 'legacy\t%s\t%s\n' "${entry%%=*}" "$(judge "$tree" "${entry#*=}")"; done
