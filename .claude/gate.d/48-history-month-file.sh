#!/usr/bin/env bash
# gate-stage: 决策变更史的条目住在它日期所在月的那一份
#
# 2026-09-12 起决策变更史按月拆档：条目住 `.claude/kb/<年-月>-decisions-history.md`，
# `decisions-history.md` 只放说明和月份表（拆档前它一份就有 7150 行、346 条）。
# 别处引一条历史，写的是「decisions-history.md 某日（其 N）」，读的人靠日期找到那个月的那一份
# ⇒ 条目住错月份、或写回 decisions-history.md，这个找法就断了，而且不会有任何别的东西报警。
#
# 文件名非得以 `decisions-history.md` 结尾：27、36、44 号阶段、lib-item-ref-status.py、relabel-item.py
# 都按这个结尾认历史文件（历史里留旧值、旧标签是写作纪律允许的），换个名字它们就会把历史当正文判红。
set -uo pipefail
cd "${1:-$(dirname "$0")/../..}" || exit 2
bad() { printf '  ✗ %s\n' "$*"; }
ok()  { printf '  ✓ %s\n' "$*"; }
howto() { printf '     → %s\n' "$*"; }

shopt -s nullglob
files=(.claude/kb/*decisions-history.md)
if ((${#files[@]} == 0)); then
  ok "没有决策变更史文件，本阶段无对象可判"
  exit 0
fi

misplaced=(); checked=0
for f in "${files[@]}"; do
  base="$(basename "$f")"
  if [[ "$base" == decisions-history.md ]]; then
    home_month=""
  elif [[ "$base" =~ ^([0-9]{4}-[0-9]{2})-decisions-history\.md$ ]]; then
    home_month="${BASH_REMATCH[1]}"
  else
    misplaced+=("$f  文件名不是 <年-月>-decisions-history.md")
    continue
  fi
  while IFS=: read -r line_number heading; do
    checked=$((checked + 1))
    entry_date="${heading:4:10}"
    if [[ -z "$home_month" ]]; then
      misplaced+=("$f:$line_number  日期 $entry_date 的条目写在了只放说明的 decisions-history.md 里")
    elif [[ "${entry_date:0:7}" != "$home_month" ]]; then
      misplaced+=("$f:$line_number  日期 $entry_date 的条目住在 $home_month 那一份")
    fi
  done < <(grep -n '^### 20[0-9][0-9]-[0-9][0-9]-[0-9][0-9]' "$f")
done

if ((${#misplaced[@]})); then
  bad "决策变更史有 ${#misplaced[@]} 处条目住错了地方"
  printf '     %s\n' "${misplaced[@]}"
  howto "条目按日期住 .claude/kb/<年-月>-decisions-history.md，放在它「## 历史版本」下的最上面；"
  howto "  那个月的文件还没有就新建，文件头照上个月那份写，并在 decisions-history.md 的月份表顶上加一行。"
  howto "decisions-history.md 只放说明和月份表：别处按「decisions-history.md 某日（其 N）」引历史，靠的就是日期对得上月份。"
  exit 1
fi
ok "查了 ${#files[@]} 份决策变更史、$checked 条条目，都住在日期对得上的那一份"
