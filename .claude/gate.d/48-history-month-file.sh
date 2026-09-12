#!/usr/bin/env bash
# gate-stage: 决策变更史的条目住在它日期所在月的那一份
#
# 2026-09-12 起决策变更史按月拆档：条目原文住 `.claude/kb/decisions-history/<年-月>.md`，
# `decisions-history.md` 放快查表（拆档前它一份就有 7150 行、347 条）。
# 别处引一条历史写的是「decisions-history.md 某日（其 N）」，读的人按日期到那个月的那一份去找
# ⇒ 条目住错月份、写回 decisions-history.md、或按月的文件放错了地方，这个找法就断了，
#   而且不会有任何别的东西报警。
set -uo pipefail
cd "${1:-$(dirname "$0")/../..}" || exit 2
bad() { printf '  ✗ %s\n' "$*"; }
ok()  { printf '  ✓ %s\n' "$*"; }
howto() { printf '     → %s\n' "$*"; }

main_file=.claude/kb/decisions-history.md
shopt -s nullglob
month_files=(.claude/kb/decisions-history/*.md)
stray_files=(.claude/kb/*-decisions-history.md)
shopt -u nullglob
files=()
[[ -f "$main_file" ]] && files+=("$main_file")
files+=("${month_files[@]}")
if ((${#files[@]} + ${#stray_files[@]} == 0)); then
  ok "没有决策变更史文件，本阶段无对象可判"
  exit 0
fi

misplaced=(); checked=0
for f in "${stray_files[@]}"; do
  misplaced+=("$f  按月的文件要放进 .claude/kb/decisions-history/，名字写成 <年-月>.md")
done
for f in "${files[@]}"; do
  base="$(basename "$f")"
  if [[ "$f" == "$main_file" ]]; then
    home_month=""
  elif [[ "$base" =~ ^([0-9]{4}-[0-9]{2})\.md$ ]]; then
    home_month="${BASH_REMATCH[1]}"
  else
    misplaced+=("$f  文件名不是 <年-月>.md")
    continue
  fi
  while IFS=: read -r line_number heading; do
    checked=$((checked + 1))
    entry_date="${heading:4:10}"
    if [[ -z "$home_month" ]]; then
      misplaced+=("$f:$line_number  日期 $entry_date 的条目写在了放快查表的 decisions-history.md 里")
    elif [[ "${entry_date:0:7}" != "$home_month" ]]; then
      misplaced+=("$f:$line_number  日期 $entry_date 的条目住在 $home_month 那一份")
    fi
  done < <(grep -n '^### 20[0-9][0-9]-[0-9][0-9]-[0-9][0-9]' "$f")
done

if ((${#misplaced[@]})); then
  bad "决策变更史有 ${#misplaced[@]} 处条目住错了地方"
  printf '     %s\n' "${misplaced[@]}"
  howto "条目原文按日期住 .claude/kb/decisions-history/<年-月>.md，放在它「## 历史版本」下的最上面；"
  howto "  那个月的文件还没有就新建，文件头照上个月那份写。"
  howto "decisions-history.md 只放快查表：别处按「decisions-history.md 某日（其 N）」引历史，靠的就是日期对得上月份。"
  exit 1
fi
ok "查了 ${#files[@]} 份决策变更史、$checked 条条目，都住在日期对得上的那一份"
