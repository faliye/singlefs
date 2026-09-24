#!/usr/bin/env bash
# gate-stage: 决策变更史的条目住在它日期所在月的那一份
#
# 2026-09-12 起决策变更史按月拆档：条目原文住 `.claude/kb/decisions-history/<年-月>.md`，
# `decisions-history.md` 放快查表（拆档前它一份就有 7150 行、347 条）。
# 别处引一条历史写的是「decisions-history.md 某日（其 N）」，读的人按日期到那个月的那一份去找
# ⇒ 条目住错月份、写回 decisions-history.md、或按月的文件放错了地方，这个找法就断了，
#   而且不会有任何别的东西报警。
#
# 决策正文（decisions.md 与 decisions/*.md）文末的「历史版本」只放一句指路、不写条目：
# 写在那里的条目不在上面那批文件里，日期对不对得上月份没人看，同一件事还成了两处记录。
# 实测（2026-09-14）：D28（挂载期承诺量） 新立时在自己文末写了一条 `### 2026-09-13`，与 2026-09 那一份的（其一）
# 说的是同一件事；decisions.md 首段当时写「把依据写进文末『历史版本』」，照着写出来的正是这个形态。
# 只查文末：D26（后台整理与放置回收） 正文里有「### 2026-09-08 三轮对抗论证」这类带日期的小节标题，那是论证，不是条目。
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
decision_files=()
[[ -f .claude/kb/decisions.md ]] && decision_files+=(.claude/kb/decisions.md)
shopt -s nullglob
decision_files+=(.claude/kb/decisions/*.md)
shopt -u nullglob
if ((${#files[@]} + ${#stray_files[@]} + ${#decision_files[@]} == 0)); then
  # 退 77：门禁记「本次未跑」，不记通过（`.claude/singlefs-ai-sop/rules/show-me-test.md`「门禁不许假装通过」）
  echo "  ! 没有决策变更史文件，也没有决策正文，本阶段无对象可判"
  exit 77
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

footer_checked=0
for f in "${decision_files[@]}"; do
  footer_checked=$((footer_checked + 1))
  while IFS=: read -r line_number heading; do
    misplaced+=("$f:$line_number  决策正文文末写了条目「${heading:4:10}」，条目原文要住 decisions-history/<年-月>.md")
  done < <(awk '/^## 历史版本/ {in_footer = 1; next} in_footer && /^### 20[0-9][0-9]-[0-9][0-9]-[0-9][0-9]/ {print FNR ":" $0}' "$f")
done

if ((${#misplaced[@]})); then
  bad "决策变更史有 ${#misplaced[@]} 处条目住错了地方"
  printf '     %s\n' "${misplaced[@]}"
  howto "条目原文按日期住 .claude/kb/decisions-history/<年-月>.md，放在它「## 历史版本」下的最上面；"
  howto "  那个月的文件还没有就新建，文件头照上个月那份写。"
  howto "decisions-history.md 只放快查表：别处按「decisions-history.md 某日（其 N）」引历史，靠的就是日期对得上月份。"
  howto "决策正文（decisions.md 与 decisions/）文末的「历史版本」只放一句指路，照别的决策文件写「Dn（简称）的历史条目集中在 decisions-history.md」；"
  howto "  文末的条目挪进它日期所在月的那一份，那一份里已经有同一件事就直接删掉，别留两处。"
  exit 1
fi
ok "查了 ${#files[@]} 份决策变更史、$checked 条条目，都住在日期对得上的那一份；$footer_checked 份决策正文的文末没写条目"
