#!/usr/bin/env bash
# Q4（第一轮 Z4-乙，第二轮辩方判「修到」）：登记之后照 .claude/rules/path-moves.md 跑 sweep-term.py --apply，records/ 照换，登记表在豁免目录里不换。
# 这一份只喂第一轮没喂过的两形：〔〕在一个旧名的中间收尾（「…超级」，那一行是「超级块」）；〔〕在一个旧名的中间起头（「块的校验…」）。对照：〔〕把旧名整个抄进去。
# 小树只放调度记录、登记表、改名表（取给的改名表那一份）与豁免表；改名用 <改名用的副本> 的 sweep-term.py，判法用 <名>= 那一版自己的 lib_abandoned_rounds.py。
# 用法：q4_term_cut.sh <名>=<那一版副本根> <改名用的副本根> <工作目录>
set -euo pipefail
label="${1%%=*}" version="${1#*=}" current="$2" work="$3"
mkdir -p "$work"
case_tree() {   # 名 轮名 记录那一行 〔〕原话
  local tree="$work/$label-$1"
  rm -rf "$tree"; mkdir -p "$tree/records" "$tree/research/prompts/$2-snapshot" "$tree/.claude/kb" "$tree/scripts/kb"
  : > "$tree/research/prompts/_$2-body.md"
  printf '%s\n' '# 调度记录' '' '## 课题' '' "- 开轮 $2：冻结副本在工作根下，快照 research/prompts/$2-snapshot/。" "$3" '- 别的活照常。' > "$tree/records/log.md"
  printf '%s\t2026-10-14\t用户叫停（出处 records/log.md 第 6 行〔%s〕），两条腿当场停掉\t不写判决\n' "$2" "$4" > "$tree/research/prompts/abandoned-rounds.tsv"
  cp "$current/.claude/kb/term-renames.md" "$tree/.claude/kb/term-renames.md"
  : > "$tree/scripts/kb/sweep-term.py"
  printf '%s\n' 'research/prompts/  # 冻结证据（照冻结副本那张表的同一行）' 'scripts/kb/sweep-term.py  # 清扫工具自己' '.claude/term-rename-exempt  # 这张表自己' '.claude/kb/term-renames.md  # 对照表自己' > "$tree/.claude/term-rename-exempt"
  echo "$tree"
}
judge() { ( cd "$1" && PYTHONDONTWRITEBYTECODE=1 python3 -B "$version/.claude/hooks/lib_abandoned_rounds.py" . 2>&1 ) | tr '\n' ' ' | python3 -c 'import sys; sys.stdout.write(sys.stdin.read()[:300])' || true; }
report() {   # 名 轮名 记录那一行 〔〕原话
  local tree before after
  tree="$(case_tree "$1" "$2" "$3" "$4")"
  before="$(judge "$tree")"
  ( cd "$tree" && PYTHONDONTWRITEBYTECODE=1 python3 -B "$current/scripts/kb/sweep-term.py" "$tree" --apply > "$work/$label-$1.apply.txt" 2>&1 ) || true
  after="$(judge "$tree")"
  printf '%s\t%s\t改名之后第 6 行：%s\t改动前：%s\t改动后：%s\n' "$label" "$1" "$(sed -n 6p "$tree/records/log.md")" "$before" "$after"
  rm -rf "$tree"
}
line='- 去向：hx-%s-r1 两条腿停掉，用户「这一轮先停掉，不写判决」，超级块的校验没跑完。'
report whole hx-whole-r1 "$(printf -- "$line" whole)" '去向：hx-whole-r1 两条腿停掉，用户「这一轮先停掉，不写判决」，超级块的校验'
report cut-end hx-cutend-r1 "$(printf -- "$line" cutend)" '去向：hx-cutend-r1 两条腿停掉，用户「这一轮先停掉，不写判决」，超级'
report cut-start hx-cutstart-r1 '- 超级块的校验没跑完：去向 hx-cutstart-r1 两条腿停掉，用户「这一轮先停掉，不写判决」。' '块的校验没跑完：去向 hx-cutstart-r1 两条腿停掉'
