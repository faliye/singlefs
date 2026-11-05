#!/usr/bin/env bash
# 先提交一份带未定项的决策、把上游设在这个提交，再另提交一次、不推：在未定项那一节之前加一节 `### 已定（2026-10-13）：丙问题`（只剩下这一种光秃秃的「已定（」标题，格不再认它），
# 不碰那条未定项；索引页的待议节同一次补了复核注。工作区另改一个无关文件。
# 本该退 77：这一格只认首行登记标题的 `—— 已定` 与分项定案的 `已定项 N：…`，光秃秃的「已定（」标题不算新增了「已定」小节，报「本次 diff 没有新增「已定」小节」。
# 弄坏开关 DOC_DECISIONS_BREAK=settled-item-heading-unrecognised 对它没有影响（本来就不认）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master . && git add -A && git commit -qm base
git branch 上游 && git branch -q --set-upstream-to=上游 master
decision=.claude/kb/decisions/981-分项定案标题样本.md
awk '/^### 未定项$/ { print "### 已定（2026-10-13）：丙问题"; print ""; print "取丙。"; print "" } { print }' "$decision" > "$decision.new" && mv "$decision.new" "$decision"
grep -q '^### 已定（2026-10-13）：丙问题$' "$decision" || { echo "setup：没插进「已定（」标题" >&2; exit 1; }
sed -i 's/^悬着的一件事，定案之前就在这里。$/悬着的一件事，2026-10-13 复核过，仍然开着。/' .claude/kb/decisions.md
grep -q '2026-10-13 复核过，仍然开着' .claude/kb/decisions.md || { echo "setup：没补上待议节的复核注" >&2; exit 1; }
git add -A && git commit -qm '定案，没推'
printf '无关的改动\n' > 无关.md
