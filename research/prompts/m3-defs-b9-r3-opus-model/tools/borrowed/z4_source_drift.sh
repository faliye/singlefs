#!/usr/bin/env bash
# Z4 喂法：叫停登记照规则写出处（这一版要〔〕原话的写原话，不要的只写行号），登记时收；之后照别的规矩改出处那份调度记录，看同一张表还收不收。
# 三种之后的改动：
#   self-quote  同一份调度记录里往后追加一行补记，抄了登记用的那段原话（主 agent 记「叫停登记已收」时顺手抄出处）
#   term-sweep  术语改名照 .claude/rules/path-moves.md 跑 scripts/kb/sweep-term.py --apply（records/ 照换，research/prompts/ 整个豁免）
#   date-once   碰过这份记录之后门禁 doc-text 的 date-once 格的出路 doc-consolidate.py migrate 删正文里重复标题的日期
# 判法用那一版自己的 .claude/hooks/lib_abandoned_rounds.py；改名与删日期用这一轮冻结副本的 sweep-term.py、doc-consolidate.py 与改名登记表（表里补上 2026-10-13 那一行，见 case_tree）。
# 用法：z4_source_drift.sh <名>=<那一版冻结副本根> <这一轮冻结副本根> <工作目录> <excerpt|line>（excerpt：出处带〔原话〕；line：只写行号）
set -euo pipefail
label="${1%%=*}" version="${1#*=}" current="$2" work="$3" form="$4"
mkdir -p "$work"
row() {   # 轮名 原话 → 登记行
  local round="$1" excerpt="$2" source="records/log.md 第 6 行"
  [[ "$form" == excerpt ]] && source="$source〔$excerpt〕"
  printf '%s\t2026-10-12\t用户叫停（出处 %s），两条腿当场停掉\t不写判决\n' "$round" "$source"
}
case_tree() {   # 名 记录第 6 行 轮名 原话
  local tree="$work/$label-$1"
  rm -rf "$tree"; mkdir -p "$tree/records" "$tree/research/prompts/$3-snapshot" "$tree/.claude/kb" "$tree/scripts/kb"
  : > "$tree/research/prompts/_$3-body.md"
  printf '%s\n' '# 调度记录' '' '## 2026-10-12' '' "- 开轮 $3：冻结副本在工作根下，快照 research/prompts/$3-snapshot/。" "$2" '- 别的活照常。' '' > "$tree/records/log.md"
  row "$3" "$4" > "$tree/research/prompts/abandoned-rounds.tsv"
  # 这一轮冻结副本的改名表还没有 2026-10-13 那一次（第一条流 → 新池新建文件流）；照主工作区那一次加的行（背景材料附录抄的 term-renames.md 表行）补在表的最后一行之后，模拟登记之后才改名
  python3 -B - "$current/.claude/kb/term-renames.md" "$tree/.claude/kb/term-renames.md" <<'PY'
import sys
source, target = sys.argv[1:3]
lines = open(source, encoding="utf-8").read().split("\n")
last = max(index for index, line in enumerate(lines) if line.startswith("| "))
lines.insert(last + 1, "| 第一条流 | 新池新建文件流 | 整串 | 中文术语：崩溃点重放的第一条负载 |")
open(target, "w", encoding="utf-8").write("\n".join(lines))
PY
  : > "$tree/scripts/kb/sweep-term.py"
  printf '%s\n' 'research/prompts/  # 冻结证据（照这一轮冻结副本那张表的同一行）' 'scripts/kb/sweep-term.py  # 清扫工具自己' '.claude/term-rename-exempt  # 这张表自己' '.claude/kb/term-renames.md  # 对照表自己' > "$tree/.claude/term-rename-exempt"
  echo "$tree"
}
judge() {
  ( cd "$1" && PYTHONDONTWRITEBYTECODE=1 python3 -B "$version/.claude/hooks/lib_abandoned_rounds.py" . 2>&1 ) | tr '\n' ' ' || true
}
report() {   # 名 树 之后的改动
  local before after
  before="$(judge "$2")"
  eval "$3"
  after="$(judge "$2")"
  printf '%s\t%s\t%s\t改动前：%s\t改动后：%s\n' "$label" "$form" "$1" "$before" "$after"
}
t="$(case_tree self-quote '- 用户「这一轮先停掉，不写判决了」。去向：hx-quote-r1 两条腿当场停掉（没交回、不写判决）' hx-quote-r1 '去向：hx-quote-r1 两条腿当场停掉（没交回、不写判决）')"
report self-quote "$t" "printf '%s\n' '- 补记：叫停登记已收进 research/prompts/abandoned-rounds.tsv，出处那一行写的是「去向：hx-quote-r1 两条腿当场停掉（没交回、不写判决）」。' >> '$t/records/log.md'"
t="$(case_tree term-sweep '- 用户「第一条流那一轮先停掉，不写判决」。去向：hx-term-r1 两条腿停掉（第一条流的崩溃枚举没跑完）' hx-term-r1 '去向：hx-term-r1 两条腿停掉（第一条流的崩溃枚举没跑完）')"
report term-sweep "$t" "( cd '$t' && python3 -B '$current/scripts/kb/sweep-term.py' '$t' --apply > '$work/$label-$form-term-sweep.apply.txt' 2>&1 ) || true"
t="$(case_tree date-once '- hx-date-r1 停掉（用户 2026-10-12 说「先停掉那一轮不写判决」）' hx-date-r1 'hx-date-r1 停掉（用户 2026-10-12 说「先停掉那一轮不写判决」）')"
report date-once "$t" "( cd '$t' && python3 -B '$current/scripts/kb/doc-consolidate.py' migrate records/log.md --out '$t/migrated.md' --force > '$work/$label-$form-date-once.migrate.txt' 2>&1 && mv '$t/migrated.md' records/log.md ) || true"
